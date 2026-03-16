use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use praxis_core::{
    doctor_workspace, init_workspace, inspect_source_input, install_source, list_workspace,
    plan_install, read_agent_file_state, remove_from_source, sync_workspace, update_workspace,
    write_agent_file_user_content, Agent, AgentFileSlot, AgentFileWriteRequest, InstallRequest,
    RemoveRequest, Scope,
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
