use anyhow::{anyhow, Context, Result};
use codex_runtime::runtime::RunProfile;
use codex_runtime::{quick_run, quick_run_with_profile};
use semver::Version;
use std::path::Path;
use std::process::Command;
use tokio::runtime::Runtime;

use crate::model::{DoctorCheck, ExternalExecutorConfig, ExternalExecutorKind};

const CODEX_RUNTIME_MIN_CLI_VERSION: &str = "0.104.0";

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

pub fn doctor_executor(config: &ExternalExecutorConfig) -> Vec<DoctorCheck> {
    match config.provider {
        ExternalExecutorKind::Disabled => vec![DoctorCheck {
            level: "info".to_string(),
            code: "external-executor-disabled".to_string(),
            message: "external executor is disabled".to_string(),
        }],
        ExternalExecutorKind::CodexRuntime => doctor_codex_runtime(config),
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

fn doctor_codex_runtime(config: &ExternalExecutorConfig) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();
    match Command::new("codex").arg("--version").output() {
        Ok(output) => {
            if !output.status.success() {
                checks.push(DoctorCheck {
                    level: "error".to_string(),
                    code: "codex-cli-version-command-failed".to_string(),
                    message: format!(
                        "failed to run `codex --version`: {}",
                        String::from_utf8_lossy(&output.stderr).trim()
                    ),
                });
                return checks;
            }

            let version_output = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let Some(version) = parse_codex_cli_version(&version_output) else {
                checks.push(DoctorCheck {
                    level: "warning".to_string(),
                    code: "codex-cli-version-unparsed".to_string(),
                    message: format!(
                        "found codex CLI but could not parse version from `{}`",
                        version_output
                    ),
                });
                return checks;
            };
            let minimum = Version::parse(CODEX_RUNTIME_MIN_CLI_VERSION)
                .expect("minimum codex runtime CLI version must parse");
            if version < minimum {
                checks.push(DoctorCheck {
                    level: "error".to_string(),
                    code: "codex-cli-version-too-old".to_string(),
                    message: format!(
                        "codex-runtime requires codex CLI >= {}; found {}",
                        minimum, version
                    ),
                });
            } else {
                checks.push(DoctorCheck {
                    level: "info".to_string(),
                    code: "codex-cli-ready".to_string(),
                    message: format!(
                        "codex CLI {} is available on PATH for codex-runtime",
                        version
                    ),
                });
            }
        }
        Err(err) => {
            checks.push(DoctorCheck {
                level: "error".to_string(),
                code: "codex-cli-missing".to_string(),
                message: format!(
                    "codex-runtime requires `codex` on PATH and `codex --version` failed: {}",
                    err
                ),
            });
            return checks;
        }
    }

    checks.push(DoctorCheck {
        level: "info".to_string(),
        code: "codex-runtime-live-check-needed".to_string(),
        message: "static readiness passed; run the live codex-runtime harness to verify local auth and real prompt execution".to_string(),
    });
    if let Some(model) = config.model.as_deref() {
        checks.push(DoctorCheck {
            level: "info".to_string(),
            code: "codex-runtime-model-configured".to_string(),
            message: format!("codex-runtime will request model `{}`", model),
        });
    }

    checks
}

fn parse_codex_cli_version(output: &str) -> Option<Version> {
    output
        .split_whitespace()
        .find_map(|token| Version::parse(token.trim_start_matches('v')).ok())
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

    #[test]
    fn parse_codex_cli_version_extracts_semver_tokens() {
        let parsed = parse_codex_cli_version("codex-cli 0.114.0").expect("parse version");
        assert_eq!(parsed, Version::parse("0.114.0").expect("expected version"));
        assert!(parse_codex_cli_version("codex-cli dev-build").is_none());
    }
}
