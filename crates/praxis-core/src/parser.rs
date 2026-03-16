use anyhow::{anyhow, bail, Context, Result};
use regex::Regex;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::model::SourceRef;

#[derive(Debug, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub compatibility: Option<String>,
    pub metadata: Option<BTreeMap<String, serde_yaml::Value>>,
}

#[derive(Debug, Default, Deserialize)]
pub struct SkillJsonSidecar {
    pub display_name: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OpenAiYaml {
    pub interface: Option<OpenAiInterface>,
}

#[derive(Debug, Default, Deserialize)]
pub struct OpenAiInterface {
    pub display_name: Option<String>,
    pub short_description: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DeckManifest {
    pub version: u32,
    pub decks: Vec<DeckManifestEntry>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DeckManifestEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub skills: Vec<String>,
}

pub fn expand_home(input: &str) -> Result<PathBuf> {
    if let Some(rest) = input.strip_prefix("~/") {
        let home = dirs::home_dir().ok_or_else(|| anyhow!("home directory not found"))?;
        return Ok(home.join(rest));
    }
    Ok(PathBuf::from(input))
}

pub fn parse_source_input(input: &str) -> Result<SourceRef> {
    let input = input.trim();

    if input.starts_with("https://github.com/") || input.starts_with("http://github.com/") {
        return parse_github_url(input);
    }

    if looks_like_owner_repo(input) && !Path::new(input).exists() {
        return parse_owner_repo(input);
    }

    Ok(SourceRef::Local {
        path: expand_home(input)?.to_string_lossy().to_string(),
    })
}

pub fn canonical_source_id(source: &SourceRef) -> String {
    match source {
        SourceRef::Github {
            owner,
            repo,
            reference,
            subdir,
        } => format!(
            "github:{}/{}@{}#{}",
            owner,
            repo,
            reference.clone().unwrap_or_else(|| "default".to_string()),
            subdir.clone().unwrap_or_else(|| "root".to_string())
        ),
        SourceRef::Local { path } => format!("local:{}", path),
    }
}

fn looks_like_owner_repo(input: &str) -> bool {
    let pathish = input.starts_with("./") || input.starts_with('/') || input.starts_with("~/");
    if pathish {
        return false;
    }
    input.split('/').count() == 2 || input.contains('@')
}

fn parse_owner_repo(input: &str) -> Result<SourceRef> {
    let (owner_repo, reference) = if let Some((left, right)) = input.rsplit_once('@') {
        if left.split('/').count() == 2 && !right.is_empty() {
            (left.to_string(), Some(right.to_string()))
        } else {
            (input.to_string(), None)
        }
    } else {
        (input.to_string(), None)
    };

    let mut parts = owner_repo.split('/');
    let owner = parts.next().unwrap_or_default().to_string();
    let repo = parts
        .next()
        .unwrap_or_default()
        .trim_end_matches(".git")
        .to_string();

    if owner.is_empty() || repo.is_empty() {
        bail!("invalid GitHub source input: {input}");
    }

    Ok(SourceRef::Github {
        owner,
        repo,
        reference,
        subdir: None,
    })
}

fn parse_github_url(input: &str) -> Result<SourceRef> {
    let input = input
        .trim_start_matches("https://github.com/")
        .trim_start_matches("http://github.com/");
    let segments: Vec<&str> = input.split('/').filter(|s| !s.is_empty()).collect();

    if segments.len() < 2 {
        bail!("invalid GitHub URL: {input}");
    }

    let owner = segments[0].to_string();
    let repo = segments[1].trim_end_matches(".git").to_string();

    let mut reference = None;
    let mut subdir = None;

    if segments.len() >= 4 && segments[2] == "tree" {
        reference = Some(segments[3].to_string());
        if segments.len() > 4 {
            subdir = Some(segments[4..].join("/"));
        }
    }

    Ok(SourceRef::Github {
        owner,
        repo,
        reference,
        subdir,
    })
}

pub fn parse_skill_frontmatter(content: &str) -> Result<(SkillFrontmatter, String)> {
    let mut lines = content.lines();

    match lines.next() {
        Some(line) if line.trim() == "---" => {}
        _ => bail!("SKILL.md is missing YAML frontmatter"),
    }

    let mut yaml = String::new();
    let mut found_closing = false;

    for line in lines.by_ref() {
        if line.trim() == "---" {
            found_closing = true;
            break;
        }
        yaml.push_str(line);
        yaml.push('\n');
    }

    if !found_closing {
        bail!("SKILL.md frontmatter is not closed");
    }

    let fm: SkillFrontmatter =
        serde_yaml::from_str(&yaml).with_context(|| "failed to parse SKILL.md YAML frontmatter")?;
    let body = lines.collect::<Vec<_>>().join("\n");
    Ok((fm, body))
}

pub fn validate_skill(name: &str, description: &str, dir_name: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let re = Regex::new(r"^[a-z0-9]+(-[a-z0-9]+)*$").expect("skill regex should compile");

    if name.is_empty() || name.len() > 64 {
        errors.push("frontmatter.name must be 1..=64 characters".to_string());
    }
    if !re.is_match(name) {
        errors.push("frontmatter.name must match ^[a-z0-9]+(-[a-z0-9]+)*$".to_string());
    }
    if dir_name != name {
        errors.push(format!(
            "skill directory '{}' does not match frontmatter.name '{}'",
            dir_name, name
        ));
    }
    if description.trim().is_empty() {
        errors.push("frontmatter.description must not be empty".to_string());
    }
    if description.len() > 1024 {
        errors.push("frontmatter.description must be <= 1024 chars".to_string());
    }

    errors
}
