use crate::model::{Agent, DeckInfo, RecipeBundle, RecipeHint, SkillInfo, SourceRef};

pub fn detect_recipe(source: &SourceRef, skills: &[SkillInfo]) -> Option<RecipeHint> {
    match source {
        SourceRef::Github { owner, repo, .. } if owner == "garrytan" && repo == "gstack" => {
            Some(gstack_recipe(skills))
        }
        _ => None,
    }
}

pub fn recipe_decks(recipe: &RecipeHint, skills: &[SkillInfo]) -> Vec<DeckInfo> {
    if recipe.key != "gstack" {
        return Vec::new();
    }

    let mut workflow_skills = Vec::new();
    for skill in skills {
        if skill.name == "gstack" {
            continue;
        }
        workflow_skills.push(skill.name.clone());
    }

    if workflow_skills.is_empty() {
        return Vec::new();
    }

    vec![DeckInfo {
        id: "workflow".to_string(),
        name: "Workflow".to_string(),
        description: "Nested slash-command workflow skills from gstack.".to_string(),
        skills: workflow_skills,
        synthesized: true,
    }]
}

fn gstack_recipe(skills: &[SkillInfo]) -> RecipeHint {
    let has_root_bundle = skills.iter().any(|skill| skill.name == "gstack");
    let bundles = if has_root_bundle {
        vec![RecipeBundle {
            id: "gstack-bundle".to_string(),
            relative_path: ".".to_string(),
            target_name: "gstack".to_string(),
            agents: vec![Agent::Claude],
            description: "Companion repo bundle required by gstack for setup and shared assets."
                .to_string(),
        }]
    } else {
        Vec::new()
    };

    RecipeHint {
        key: "gstack".to_string(),
        label: "gstack".to_string(),
        description: "Bundle-aware install semantics for Garry Tan's Claude workflow skills."
            .to_string(),
        bundles,
        notes: vec![
            "After install, run `./setup` inside `.claude/skills/gstack` if you want the browser binary and command registration rebuilt.".to_string(),
            "The bundle copy is companion state; atomic skill cards remain installable one-by-one.".to_string(),
        ],
        recommended_agent_file_templates: vec!["claude-project-root".to_string()],
    }
}
