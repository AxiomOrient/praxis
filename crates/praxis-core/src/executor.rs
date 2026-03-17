use anyhow::{anyhow, Context, Result};
use codex_runtime::runtime::RunProfile;
use codex_runtime::{quick_run, quick_run_with_profile};
use std::path::Path;
use tokio::runtime::Runtime;

use crate::model::{ExternalExecutorConfig, ExternalExecutorKind};

pub fn run_prompt(
    workdir: &Path,
    prompt: &str,
    executor: Option<&ExternalExecutorConfig>,
) -> Result<String> {
    let executor = executor.cloned().unwrap_or_default();
    if !executor.is_enabled() {
        return Err(anyhow!("external executor is disabled"));
    }

    match executor.provider {
        ExternalExecutorKind::Disabled => Err(anyhow!("external executor is disabled")),
        ExternalExecutorKind::CodexRuntime => run_codex_runtime_prompt(workdir, prompt, &executor),
    }
}

fn run_codex_runtime_prompt(
    workdir: &Path,
    prompt: &str,
    executor: &ExternalExecutorConfig,
) -> Result<String> {
    let cwd = workdir
        .to_str()
        .ok_or_else(|| anyhow!("workdir is not valid UTF-8: {}", workdir.display()))?
        .to_string();
    let prompt = prompt.to_string();
    let runtime = Runtime::new().context("create tokio runtime for codex-runtime")?;
    let result = runtime.block_on(async move {
        if let Some(model) = executor.model.as_deref() {
            quick_run_with_profile(cwd, prompt, RunProfile::default().with_model(model)).await
        } else {
            quick_run(cwd, prompt).await
        }
    });
    let response = result.context("run codex-runtime prompt")?;
    Ok(response.assistant_text.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn disabled_executor_rejects_prompt_execution() {
        let temp = tempdir().expect("tempdir");
        let err = run_prompt(temp.path(), "hello", None).expect_err("disabled should reject");
        assert!(err.to_string().contains("disabled"));
    }

    #[test]
    fn config_reports_enabled_only_for_codex_runtime() {
        let disabled = ExternalExecutorConfig::default();
        let enabled = ExternalExecutorConfig {
            provider: ExternalExecutorKind::CodexRuntime,
            model: Some("gpt-5-codex".to_string()),
        };

        assert!(!disabled.is_enabled());
        assert!(enabled.is_enabled());
    }
}
