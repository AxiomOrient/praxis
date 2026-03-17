use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use praxis_core::{
    benchmark_source, create_draft, doctor_workspace, fork_draft, init_workspace,
    inspect_source_input, install_source, list_workspace, plan_install, preview_draft,
    promote_draft, read_agent_file_state, remove_from_source, submit_human_review_action,
    sync_workspace, update_draft, update_workspace, write_agent_file_user_content, Agent,
    AgentFileSlot, AgentFileWriteRequest, BenchmarkRunRequest, CreateDraftRequest,
    DraftPreviewRequest, DraftUpdateRequest, ForkDraftRequest, HumanReviewRequest,
    InstallRequest, PromoteDraftRequest, RemoveRequest, Scope,
};

#[derive(Debug, Parser)]
#[command(
    name = "praxis",
    about = "GitHub-first manager for agent skills and guidance"
)]
struct Cli {
    #[arg(long, value_enum, default_value_t = ScopeArg::Repo)]
    scope: ScopeArg,

    #[arg(long)]
    root: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
enum ScopeArg {
    Repo,
    User,
}

impl From<ScopeArg> for Scope {
    fn from(value: ScopeArg) -> Self {
        match value {
            ScopeArg::Repo => Scope::Repo,
            ScopeArg::User => Scope::User,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum AgentArg {
    Codex,
    Claude,
    Gemini,
}

impl From<AgentArg> for Agent {
    fn from(value: AgentArg) -> Self {
        match value {
            AgentArg::Codex => Agent::Codex,
            AgentArg::Claude => Agent::Claude,
            AgentArg::Gemini => Agent::Gemini,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum AgentFileSlotArg {
    CodexUserRoot,
    CodexUserOverride,
    CodexProjectRoot,
    CodexProjectOverride,
    ClaudeUserRoot,
    ClaudeProjectRoot,
    ClaudeProjectDot,
    GeminiUserRoot,
    GeminiProjectRoot,
}

impl From<AgentFileSlotArg> for AgentFileSlot {
    fn from(value: AgentFileSlotArg) -> Self {
        match value {
            AgentFileSlotArg::CodexUserRoot => AgentFileSlot::CodexUserRoot,
            AgentFileSlotArg::CodexUserOverride => AgentFileSlot::CodexUserOverride,
            AgentFileSlotArg::CodexProjectRoot => AgentFileSlot::CodexProjectRoot,
            AgentFileSlotArg::CodexProjectOverride => AgentFileSlot::CodexProjectOverride,
            AgentFileSlotArg::ClaudeUserRoot => AgentFileSlot::ClaudeUserRoot,
            AgentFileSlotArg::ClaudeProjectRoot => AgentFileSlot::ClaudeProjectRoot,
            AgentFileSlotArg::ClaudeProjectDot => AgentFileSlot::ClaudeProjectDot,
            AgentFileSlotArg::GeminiUserRoot => AgentFileSlot::GeminiUserRoot,
            AgentFileSlotArg::GeminiProjectRoot => AgentFileSlot::GeminiProjectRoot,
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    Inspect {
        source: String,
    },
    Plan {
        source: String,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        deck: Vec<String>,
        #[arg(long)]
        skill: Vec<String>,
        #[arg(long)]
        exclude_skill: Vec<String>,
        #[arg(long)]
        agent_file_template: Vec<String>,
        #[arg(long, value_enum)]
        agent: Vec<AgentArg>,
    },
    Install {
        source: String,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        deck: Vec<String>,
        #[arg(long)]
        skill: Vec<String>,
        #[arg(long)]
        exclude_skill: Vec<String>,
        #[arg(long)]
        agent_file_template: Vec<String>,
        #[arg(long, value_enum)]
        agent: Vec<AgentArg>,
    },
    Remove {
        source: String,
        #[arg(long)]
        deck: Vec<String>,
        #[arg(long)]
        skill: Vec<String>,
        #[arg(long)]
        agent_file_template: Vec<String>,
        #[arg(long)]
        all: bool,
    },
    List,
    Sync,
    Update,
    Doctor,
    Benchmark {
        #[command(subcommand)]
        command: BenchmarkCommand,
    },
    Create {
        #[command(subcommand)]
        command: CreateCommand,
    },
    Guidance {
        #[command(subcommand)]
        command: GuidanceCommand,
    },
}

#[derive(Debug, Subcommand)]
enum GuidanceCommand {
    Show {
        #[arg(long, value_enum)]
        slot: Option<AgentFileSlotArg>,
    },
    Set {
        #[arg(long, value_enum)]
        slot: AgentFileSlotArg,
        #[arg(long)]
        file: Option<String>,
        #[arg(long)]
        content: Option<String>,
    },
    Paths,
}

#[derive(Debug, Subcommand)]
enum BenchmarkCommand {
    Run {
        suite: String,
        source: String,
        #[arg(long, default_value = "deterministic")]
        mode: String,
    },
    Review {
        run_id: String,
        #[arg(long)]
        decision: String,
        #[arg(long, default_value = "")]
        note: String,
    },
}

#[derive(Debug, Subcommand)]
enum CreateCommand {
    Skill {
        name: String,
        #[arg(long, default_value = "Draft created from Praxis create flow.")]
        description: String,
        #[arg(long, default_value = "skill")]
        preset: String,
    },
    Preview {
        draft_id: String,
    },
    Promote {
        draft_id: String,
        #[arg(long)]
        destination_root: Option<String>,
    },
    Fork {
        source: String,
        skill_name: String,
        #[arg(long)]
        draft_name: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },
    Write {
        draft_id: String,
        path: String,
        #[arg(long)]
        content: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let scope: Scope = cli.scope.into();

    match cli.command {
        Command::Init => print_json(&init_workspace(scope, cli.root)?),
        Command::Inspect { source } => print_json(&inspect_source_input(scope, cli.root, &source)?),
        Command::Plan {
            source,
            all,
            deck,
            skill,
            exclude_skill,
            agent_file_template,
            agent,
        } => {
            let req = InstallRequest {
                scope,
                root: cli.root,
                source,
                all,
                decks: deck,
                skills: skill,
                exclude_skills: exclude_skill,
                agent_file_templates: agent_file_template,
                targets: agent.into_iter().map(Into::into).collect(),
            };
            print_json(&plan_install(req)?)
        }
        Command::Install {
            source,
            all,
            deck,
            skill,
            exclude_skill,
            agent_file_template,
            agent,
        } => {
            let req = InstallRequest {
                scope,
                root: cli.root,
                source,
                all,
                decks: deck,
                skills: skill,
                exclude_skills: exclude_skill,
                agent_file_templates: agent_file_template,
                targets: agent.into_iter().map(Into::into).collect(),
            };
            print_json(&install_source(req)?)
        }
        Command::Remove {
            source,
            deck,
            skill,
            agent_file_template,
            all,
        } => {
            let req = RemoveRequest {
                scope,
                root: cli.root,
                source,
                decks: deck,
                skills: skill,
                agent_file_templates: agent_file_template,
                remove_all: all,
            };
            print_json(&remove_from_source(req)?)
        }
        Command::List => print_json(&list_workspace(scope, cli.root)?),
        Command::Sync => print_json(&sync_workspace(scope, cli.root)?),
        Command::Update => print_json(&update_workspace(scope, cli.root)?),
        Command::Doctor => print_json(&doctor_workspace(scope, cli.root)?),
        Command::Benchmark { command } => match command {
            BenchmarkCommand::Run { suite, source, mode } => print_json(&benchmark_source(
                BenchmarkRunRequest {
                    scope,
                    root: cli.root,
                    suite_id: suite,
                    source,
                    mode: Some(mode),
                },
            )?),
            BenchmarkCommand::Review { run_id, decision, note } => print_json(
                &submit_human_review_action(HumanReviewRequest {
                    scope,
                    root: cli.root,
                    run_id,
                    decision,
                    note,
                })?,
            ),
        },
        Command::Create { command } => match command {
            CreateCommand::Skill {
                name,
                description,
                preset,
            } => print_json(&create_draft(CreateDraftRequest {
                scope,
                root: cli.root,
                name,
                description,
                preset,
            })?),
            CreateCommand::Preview { draft_id } => print_json(&preview_draft(DraftPreviewRequest {
                scope,
                root: cli.root,
                draft_id,
            })?),
            CreateCommand::Promote {
                draft_id,
                destination_root,
            } => print_json(&promote_draft(PromoteDraftRequest {
                scope,
                root: cli.root,
                draft_id,
                destination_root,
            })?),
            CreateCommand::Fork {
                source,
                skill_name,
                draft_name,
                description,
            } => print_json(&fork_draft(ForkDraftRequest {
                scope,
                root: cli.root,
                source,
                skill_name,
                draft_name,
                description,
            })?),
            CreateCommand::Write {
                draft_id,
                path,
                content,
            } => print_json(&update_draft(DraftUpdateRequest {
                scope,
                root: cli.root,
                draft_id,
                relative_path: path,
                content,
            })?),
        },
        Command::Guidance { command } => match command {
            GuidanceCommand::Show { slot } => {
                let snapshot = read_agent_file_state(scope, cli.root)?;
                if let Some(slot_arg) = slot {
                    let needle: AgentFileSlot = slot_arg.into();
                    let filtered = snapshot
                        .slots
                        .into_iter()
                        .filter(|s| s.slot == needle)
                        .collect::<Vec<_>>();
                    print_json(&filtered)
                } else {
                    print_json(&snapshot)
                }
            }
            GuidanceCommand::Set {
                slot,
                file,
                content,
            } => {
                let next_content = if let Some(path) = file {
                    std::fs::read_to_string(path)?
                } else {
                    content.unwrap_or_default()
                };
                print_json(&write_agent_file_user_content(AgentFileWriteRequest {
                    scope,
                    root: cli.root,
                    slot: slot.into(),
                    content: next_content,
                })?)
            }
            GuidanceCommand::Paths => {
                let snapshot = list_workspace(scope, cli.root)?;
                print_json(&snapshot.targets)
            }
        },
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
