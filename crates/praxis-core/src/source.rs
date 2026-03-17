use anyhow::{anyhow, bail, Context, Result};
use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use tar::Archive;

use crate::model::{
    AgentFileSlot, AgentFileTemplate, AgentFileTemplateOrigin, DeckInfo, SkillInfo, SourceCatalog,
    SourceRef,
};
use crate::parser::{
    canonical_source_id, parse_skill_frontmatter, validate_skill, DeckManifest, OpenAiYaml,
    SkillJsonSidecar,
};
use crate::recipes::{detect_recipe, recipe_decks};

const DEFAULT_DISCOVERED_TEMPLATE_PRIORITY: u32 = 100;

#[derive(Debug)]
struct PreparedSource {
    checkout_root: PathBuf,
    resolved_reference: Option<String>,
    source_hash: String,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoInfo {
    default_branch: String,
}

pub fn scan_source(source: &SourceRef, cache_dir: &Path) -> Result<SourceCatalog> {
    fs::create_dir_all(cache_dir)
        .with_context(|| format!("failed to create cache dir {}", cache_dir.display()))?;

    let prepared = prepare_source(source, cache_dir)?;
    let scan_root = &prepared.checkout_root;

    let mut warnings = Vec::new();
    let mut notes = Vec::new();
    let mut skills = Vec::new();

    let search_roots = discover_search_roots(scan_root)?;
    for root in &search_roots {
        collect_skills(scan_root, root, &mut skills, &mut warnings)?;
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    skills.dedup_by(|a, b| a.name == b.name && a.relative_path == b.relative_path);

    let agent_file_templates = discover_agent_file_templates(scan_root)?;
    let recipe = detect_recipe(source, &skills);
    let mut decks = discover_decks(scan_root, &skills, &mut warnings)?;
    if let Some(recipe) = &recipe {
        decks.extend(recipe_decks(recipe, &skills));
        notes.extend(recipe.notes.clone());
    }
    normalize_decks(&mut decks);

    let label = match source {
        SourceRef::Github { owner, repo, .. } => format!("{owner}/{repo}"),
        SourceRef::Local { path } => Path::new(path)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone()),
    };

    Ok(SourceCatalog {
        source_id: canonical_source_id(source),
        label,
        source: source.clone(),
        checkout_root: scan_root.to_string_lossy().to_string(),
        resolved_reference: prepared.resolved_reference,
        source_hash: prepared.source_hash,
        decks,
        skills,
        agent_file_templates,
        recipe,
        warnings,
        notes,
    })
}

fn prepare_source(source: &SourceRef, cache_dir: &Path) -> Result<PreparedSource> {
    match source {
        SourceRef::Local { path } => {
            let root = fs::canonicalize(path)
                .with_context(|| format!("failed to resolve local path '{}'", path))?;
            let source_hash = hash_directory(&root)?;
            Ok(PreparedSource {
                checkout_root: root,
                resolved_reference: None,
                source_hash,
            })
        }
        SourceRef::Github {
            owner,
            repo,
            reference,
            subdir,
        } => prepare_github_source(owner, repo, reference.clone(), subdir.clone(), cache_dir),
    }
}

fn prepare_github_source(
    owner: &str,
    repo: &str,
    reference: Option<String>,
    subdir: Option<String>,
    cache_dir: &Path,
) -> Result<PreparedSource> {
    let client = Client::builder()
        .user_agent("praxis/1.0")
        .build()
        .context("failed to create HTTP client")?;
    let cache_is_reusable = is_immutable_ref(reference.as_deref());

    let resolved_ref = match reference {
        Some(r) => r,
        None => {
            let repo_url = format!("https://api.github.com/repos/{owner}/{repo}");
            let info = client
                .get(repo_url)
                .send()
                .context("failed to resolve GitHub default branch")?
                .error_for_status()
                .context("GitHub repo lookup returned non-success status")?
                .json::<GitHubRepoInfo>()
                .context("failed to parse GitHub repo metadata")?;
            info.default_branch
        }
    };

    let tarball_url = format!("https://api.github.com/repos/{owner}/{repo}/tarball/{resolved_ref}");
    let cache_root = cache_dir.join(source_cache_key(
        owner,
        repo,
        &resolved_ref,
        subdir.as_deref(),
    ));
    let unpack_root = cache_root.join("unpacked");

    if unpack_root.is_dir() && cache_is_reusable {
        let repo_root = first_directory(&unpack_root)?
            .ok_or_else(|| anyhow!("cached GitHub source is missing a root directory"))?;
        let checkout_root = if let Some(subdir) = subdir.clone() {
            let path = repo_root.join(subdir);
            if !path.is_dir() {
                bail!("requested subdir does not exist inside cached source");
            }
            path
        } else {
            repo_root
        };

        let source_hash = hash_directory(&checkout_root)?;
        return Ok(PreparedSource {
            checkout_root,
            resolved_reference: Some(resolved_ref),
            source_hash,
        });
    }

    let bytes = client
        .get(tarball_url)
        .send()
        .context("failed to download GitHub tarball")?
        .error_for_status()
        .context("GitHub tarball request returned non-success status")?
        .bytes()
        .context("failed to read GitHub tarball response")?
        .to_vec();

    if cache_root.exists() {
        fs::remove_dir_all(&cache_root)?;
    }
    fs::create_dir_all(&cache_root)?;
    fs::create_dir_all(&unpack_root)?;

    let gz = GzDecoder::new(Cursor::new(bytes));
    let mut archive = Archive::new(gz);
    safe_unpack_tar(&mut archive, &unpack_root)?;

    let repo_root = first_directory(&unpack_root)?
        .ok_or_else(|| anyhow!("GitHub tarball did not unpack into a root directory"))?;
    let checkout_root = if let Some(subdir) = subdir {
        let path = repo_root.join(subdir);
        if !path.is_dir() {
            bail!("requested subdir does not exist inside source");
        }
        path
    } else {
        repo_root
    };

    let source_hash = hash_directory(&checkout_root)?;
    Ok(PreparedSource {
        checkout_root,
        resolved_reference: Some(resolved_ref),
        source_hash,
    })
}

fn is_immutable_ref(reference: Option<&str>) -> bool {
    matches!(
        reference,
        Some(value)
            if (7..=40).contains(&value.len())
                && value.chars().all(|ch| ch.is_ascii_hexdigit())
    )
}

fn safe_unpack_tar(archive: &mut Archive<GzDecoder<Cursor<Vec<u8>>>>, dest: &Path) -> Result<()> {
    for entry in archive
        .entries()
        .context("failed to enumerate tar entries")?
    {
        let mut entry = entry?;
        let raw_path = entry.path()?.into_owned();
        let safe_rel = strip_first_component(&raw_path)?;
        if safe_rel.as_os_str().is_empty() {
            continue;
        }

        let target = dest.join(&safe_rel);
        ensure_within(dest, &target)?;

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }

        let entry_type = entry.header().entry_type();
        if entry_type.is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }

        if entry_type.is_symlink() || entry_type.is_hard_link() {
            bail!(
                "refusing tarball entry with link type: {}",
                raw_path.display()
            );
        }

        entry.unpack(&target)?;
    }

    Ok(())
}

fn strip_first_component(path: &Path) -> Result<PathBuf> {
    let mut comps = path.components();
    comps.next();
    let mut out = PathBuf::new();
    for comp in comps {
        match comp {
            Component::Normal(part) => out.push(part),
            Component::CurDir => {}
            _ => bail!("unsafe tar entry path: {}", path.display()),
        }
    }
    Ok(out)
}

fn ensure_within(root: &Path, target: &Path) -> Result<()> {
    let root = fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let parent = target.parent().unwrap_or(root.as_path());
    let parent = fs::canonicalize(parent).unwrap_or_else(|_| parent.to_path_buf());
    if !parent.starts_with(&root) {
        bail!("tar extraction attempted to escape destination");
    }
    Ok(())
}

fn discover_search_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut roots = Vec::new();

    if root.join("SKILL.md").is_file() {
        roots.push(root.to_path_buf());
    }

    for candidate in [
        root.join("skills"),
        root.join(".agents").join("skills"),
        root.join(".claude").join("skills"),
    ] {
        if candidate.is_dir() {
            roots.push(candidate);
        }
    }

    if roots.is_empty() {
        roots.push(root.to_path_buf());
    }

    roots.sort();
    roots.dedup();
    Ok(roots)
}

fn first_directory(dir: &Path) -> Result<Option<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to list {}", dir.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs.into_iter().next())
}

fn collect_skills(
    scan_root: &Path,
    dir: &Path,
    out: &mut Vec<SkillInfo>,
    warnings: &mut Vec<String>,
) -> Result<()> {
    if should_skip_dir(dir) {
        return Ok(());
    }

    let skill_md = dir.join("SKILL.md");
    if skill_md.is_file() {
        match scan_skill_root(dir, scan_root) {
            Ok(skill) => out.push(skill),
            Err(err) => warnings.push(format!("{}: {}", dir.display(), err)),
        }
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to list {}", dir.display()))? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            collect_skills(scan_root, &entry.path(), out, warnings)?;
        }
    }

    Ok(())
}

pub(crate) fn should_skip_dir(dir: &Path) -> bool {
    matches!(
        dir.file_name().and_then(|n| n.to_str()),
        Some(".git") | Some("node_modules") | Some("target") | Some(".praxis") | Some(".telos")
    )
}

fn scan_skill_root(dir: &Path, root: &Path) -> Result<SkillInfo> {
    let skill_md_path = dir.join("SKILL.md");
    let content = fs::read_to_string(&skill_md_path)
        .with_context(|| format!("failed to read {}", skill_md_path.display()))?;
    let (fm, _body) = parse_skill_frontmatter(&content)?;

    let dir_name = dir
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or_else(|| anyhow!("invalid skill root path {}", dir.display()))?;

    let errors = validate_skill(&fm.name, &fm.description, &dir_name);
    if !errors.is_empty() {
        bail!(errors.join("; "));
    }

    let skill_json = read_skill_json(dir).unwrap_or_default();
    let openai_yaml = read_openai_yaml(dir).unwrap_or_default();

    let relative_path = dir
        .strip_prefix(root)
        .unwrap_or(dir)
        .to_string_lossy()
        .replace('\\', "/");

    let root_component = relative_path
        .split('/')
        .next()
        .unwrap_or(&relative_path)
        .to_string();

    let display_name = skill_json.display_name.or_else(|| {
        openai_yaml
            .interface
            .as_ref()
            .and_then(|i| i.display_name.clone())
    });

    let description = openai_yaml
        .interface
        .as_ref()
        .and_then(|i| i.short_description.clone())
        .unwrap_or(fm.description);

    Ok(SkillInfo {
        name: fm.name,
        description,
        relative_path,
        root_component,
        display_name,
        category: skill_json.category,
        tags: skill_json.tags.unwrap_or_default(),
    })
}

fn read_skill_json(dir: &Path) -> Result<SkillJsonSidecar> {
    let path = dir.join("skill.json");
    if !path.is_file() {
        return Ok(SkillJsonSidecar::default());
    }
    let raw = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&raw)?)
}

fn read_openai_yaml(dir: &Path) -> Result<OpenAiYaml> {
    let path = dir.join("agents").join("openai.yaml");
    if !path.is_file() {
        return Ok(OpenAiYaml::default());
    }
    let raw = fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&raw)?)
}

fn discover_agent_file_templates(root: &Path) -> Result<Vec<AgentFileTemplate>> {
    let mut templates = Vec::new();

    // Auto-discovered templates currently use a single default priority band.
    // If source manifests grow explicit template priorities, discovery should
    // hydrate those values here instead of overriding them.

    let codex_override = root.join("AGENTS.override.md");
    if codex_override.is_file() {
        templates.push(AgentFileTemplate {
            id: "codex-project-override".to_string(),
            title: "Codex override agent file".to_string(),
            description: "Higher-priority Codex project agent file content.".to_string(),
            relative_path: "AGENTS.override.md".to_string(),
            slots: vec![AgentFileSlot::CodexProjectOverride],
            priority: DEFAULT_DISCOVERED_TEMPLATE_PRIORITY,
            origin: AgentFileTemplateOrigin::Discovered,
        });
    }

    for candidate in [root.join("AGENTS.md"), root.join("AGENT.md")] {
        if candidate.is_file() {
            let rel = candidate
                .strip_prefix(root)
                .unwrap_or(&candidate)
                .to_string_lossy()
                .replace('\\', "/");
            templates.push(AgentFileTemplate {
                id: "codex-project-root".to_string(),
                title: "Codex root agent file".to_string(),
                description: "Base AGENTS.md agent file content for Codex.".to_string(),
                relative_path: rel,
                slots: vec![AgentFileSlot::CodexProjectRoot],
                priority: DEFAULT_DISCOVERED_TEMPLATE_PRIORITY,
                origin: AgentFileTemplateOrigin::Discovered,
            });
            break;
        }
    }

    let claude_root = root.join("CLAUDE.md");
    if claude_root.is_file() {
        templates.push(AgentFileTemplate {
            id: "claude-project-root".to_string(),
            title: "Claude root agent file".to_string(),
            description: "Project Claude agent file content at repository root.".to_string(),
            relative_path: "CLAUDE.md".to_string(),
            slots: vec![AgentFileSlot::ClaudeProjectRoot],
            priority: DEFAULT_DISCOVERED_TEMPLATE_PRIORITY,
            origin: AgentFileTemplateOrigin::Discovered,
        });
    }

    let claude_dot = root.join(".claude").join("CLAUDE.md");
    if claude_dot.is_file() {
        templates.push(AgentFileTemplate {
            id: "claude-project-dot".to_string(),
            title: "Claude project agent file".to_string(),
            description: "Project Claude agent file content under .claude/.".to_string(),
            relative_path: ".claude/CLAUDE.md".to_string(),
            slots: vec![AgentFileSlot::ClaudeProjectDot],
            priority: DEFAULT_DISCOVERED_TEMPLATE_PRIORITY,
            origin: AgentFileTemplateOrigin::Discovered,
        });
    }

    Ok(templates)
}

fn discover_decks(
    root: &Path,
    skills: &[SkillInfo],
    warnings: &mut Vec<String>,
) -> Result<Vec<DeckInfo>> {
    let manifest_path = root.join("skills.deck.json");
    if manifest_path.is_file() {
        let raw = fs::read_to_string(&manifest_path)?;
        let manifest: DeckManifest = serde_json::from_str(&raw)
            .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

        let skill_names: BTreeSet<String> = skills.iter().map(|s| s.name.clone()).collect();
        let mut decks = Vec::new();

        for entry in manifest.decks {
            let missing: Vec<String> = entry
                .skills
                .iter()
                .filter(|name| !skill_names.contains(*name))
                .cloned()
                .collect();
            if !missing.is_empty() {
                warnings.push(format!(
                    "deck '{}' references missing skills: {}",
                    entry.id,
                    missing.join(", ")
                ));
                continue;
            }

            decks.push(DeckInfo {
                id: entry.id,
                name: entry.name,
                description: entry.description,
                skills: entry.skills,
                synthesized: false,
            });
        }

        return Ok(decks);
    }

    let mut decks = Vec::new();

    if !skills.is_empty() {
        decks.push(DeckInfo {
            id: "all".to_string(),
            name: "All".to_string(),
            description: "All discovered skills in this source.".to_string(),
            skills: skills.iter().map(|s| s.name.clone()).collect(),
            synthesized: true,
        });
    }

    let mut categories: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for skill in skills {
        if let Some(category) = &skill.category {
            categories
                .entry(category.clone())
                .or_default()
                .push(skill.name.clone());
        }
    }

    for (category, names) in categories {
        if names.is_empty() {
            continue;
        }
        decks.push(DeckInfo {
            id: category.clone(),
            name: category.replace('-', " ").to_uppercase(),
            description: format!(
                "Category deck synthesized from skill.json category '{}'.",
                category
            ),
            skills: names,
            synthesized: true,
        });
    }

    let mut prefixes: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for skill in skills {
        if let Some((prefix, _rest)) = skill.name.split_once('-') {
            prefixes
                .entry(prefix.to_string())
                .or_default()
                .push(skill.name.clone());
        }
    }

    for (prefix, names) in prefixes {
        if names.len() < 2 {
            continue;
        }
        decks.push(DeckInfo {
            id: prefix.clone(),
            name: prefix.replace('-', " ").to_uppercase(),
            description: format!(
                "Prefix deck synthesized from skill names starting with '{}-'.",
                prefix
            ),
            skills: names,
            synthesized: true,
        });
    }

    Ok(decks)
}

fn normalize_decks(decks: &mut Vec<DeckInfo>) {
    decks.sort_by(|a, b| a.id.cmp(&b.id));
    decks.dedup_by(|a, b| a.id == b.id);
    for deck in decks.iter_mut() {
        deck.skills.sort();
        deck.skills.dedup();
    }
}

pub(crate) fn source_cache_key(
    owner: &str,
    repo: &str,
    reference: &str,
    subdir: Option<&str>,
) -> String {
    let mut hasher = Sha256::new();
    hasher.update(owner.as_bytes());
    hasher.update(repo.as_bytes());
    hasher.update(reference.as_bytes());
    hasher.update(subdir.unwrap_or("root").as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hash_directory(dir: &Path) -> Result<String> {
    let mut files = Vec::new();
    collect_files(dir, dir, &mut files)?;
    files.sort();

    let mut hasher = Sha256::new();
    for rel in &files {
        let abs = dir.join(rel);
        let bytes = fs::read(&abs).with_context(|| format!("failed to read {}", abs.display()))?;
        hasher.update(rel.as_bytes());
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(&bytes);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to list {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let ty = entry.file_type()?;
        if ty.is_symlink() {
            bail!("refusing symlinked file while hashing: {}", path.display());
        }
        if ty.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            collect_files(root, &path, out)?;
        } else if ty.is_file() {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push(rel);
        }
    }
    Ok(())
}
