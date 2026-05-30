use codex_utils_absolute_path::AbsolutePathBuf;
use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

pub(crate) const PROVIDERS_RELATIVE_DIR: &[&str] = &[".agents", "codex", "skill-root-providers.d"];
pub(crate) const PROVIDER_TIMEOUT_MS: u64 = 1000;

#[derive(Debug, Serialize, PartialEq, Eq)]
pub(crate) struct SkillRootProviderContext {
    schema_version: u8,
    cwd: String,
    home: String,
    codex_home: String,
    session_source: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SkillRootProviderOutput {
    #[serde(default)]
    roots: Vec<SkillRootProviderRoot>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SkillRootProviderRoot {
    path: String,
    #[serde(default = "default_enabled")]
    enabled: bool,
    #[serde(default, rename = "label")]
    _label: Option<String>,
}

fn default_enabled() -> bool {
    true
}

pub(crate) fn build_context(
    cwd: &AbsolutePathBuf,
    home_dir: &AbsolutePathBuf,
    codex_home: &AbsolutePathBuf,
) -> SkillRootProviderContext {
    SkillRootProviderContext {
        schema_version: 1,
        cwd: cwd.as_path().to_string_lossy().into_owned(),
        home: home_dir.as_path().to_string_lossy().into_owned(),
        codex_home: codex_home.as_path().to_string_lossy().into_owned(),
        session_source: "tui".to_string(),
    }
}

pub(crate) fn providers_dir(home_dir: &AbsolutePathBuf) -> AbsolutePathBuf {
    PROVIDERS_RELATIVE_DIR
        .iter()
        .fold(home_dir.clone(), |path, component| path.join(component))
}

pub(crate) async fn load_provider_roots(
    cwd: &AbsolutePathBuf,
    home_dir: Option<&AbsolutePathBuf>,
    codex_home: Option<&AbsolutePathBuf>,
) -> Vec<AbsolutePathBuf> {
    let (Some(home_dir), Some(codex_home)) = (home_dir, codex_home) else {
        return Vec::new();
    };

    let provider_dir = providers_dir(home_dir);
    let mut read_dir = match tokio::fs::read_dir(provider_dir.as_path()).await {
        Ok(read_dir) => read_dir,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Vec::new(),
        Err(err) => {
            tracing::warn!(
                path = %provider_dir.display(),
                "failed to read skill root providers directory: {err}"
            );
            return Vec::new();
        }
    };

    let mut providers = Vec::new();
    loop {
        let entry = match read_dir.next_entry().await {
            Ok(Some(entry)) => entry,
            Ok(None) => break,
            Err(err) => {
                tracing::warn!(
                    path = %provider_dir.display(),
                    "failed to read skill root provider entry: {err}"
                );
                continue;
            }
        };

        let file_type = match entry.file_type().await {
            Ok(file_type) => file_type,
            Err(err) => {
                tracing::warn!(
                    path = %entry.path().display(),
                    "failed to read skill root provider file type: {err}"
                );
                continue;
            }
        };
        if file_type.is_file() {
            providers.push(entry.path());
        }
    }
    providers.sort();

    let context = build_context(cwd, home_dir, codex_home);
    let mut roots = Vec::new();
    for provider in providers {
        roots.extend(run_provider(&provider, &context).await);
    }
    roots
}

async fn run_provider(provider: &Path, context: &SkillRootProviderContext) -> Vec<AbsolutePathBuf> {
    let context_json = match serde_json::to_vec(context) {
        Ok(context_json) => context_json,
        Err(err) => {
            tracing::warn!(
                provider = %provider.display(),
                "failed to serialize skill root provider context: {err}"
            );
            return Vec::new();
        }
    };

    let mut command = Command::new(provider);
    command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            tracing::warn!(
                provider = %provider.display(),
                "failed to spawn skill root provider: {err}"
            );
            return Vec::new();
        }
    };

    if let Some(mut stdin) = child.stdin.take()
        && let Err(err) = stdin.write_all(&context_json).await
    {
        let _ = child.kill().await;
        tracing::warn!(
            provider = %provider.display(),
            "failed to write skill root provider stdin: {err}"
        );
        return Vec::new();
    }

    let output = match timeout(
        Duration::from_millis(PROVIDER_TIMEOUT_MS),
        child.wait_with_output(),
    )
    .await
    {
        Ok(Ok(output)) => output,
        Ok(Err(err)) => {
            tracing::warn!(
                provider = %provider.display(),
                "failed to wait for skill root provider: {err}"
            );
            return Vec::new();
        }
        Err(_) => {
            tracing::warn!(
                provider = %provider.display(),
                timeout_ms = PROVIDER_TIMEOUT_MS,
                "skill root provider timed out"
            );
            return Vec::new();
        }
    };

    if !output.status.success() {
        tracing::warn!(
            provider = %provider.display(),
            status = %output.status,
            stderr = %brief_stderr(&output.stderr),
            "skill root provider exited unsuccessfully"
        );
        return Vec::new();
    }

    parse_output(&output.stdout, provider)
}

pub(crate) fn parse_output(stdout: &[u8], provider: &Path) -> Vec<AbsolutePathBuf> {
    let output = match serde_json::from_slice::<SkillRootProviderOutput>(stdout) {
        Ok(output) => output,
        Err(err) => {
            tracing::warn!(
                provider = %provider.display(),
                "failed to parse skill root provider output: {err}"
            );
            return Vec::new();
        }
    };

    output
        .roots
        .into_iter()
        .filter_map(|root| {
            let SkillRootProviderRoot {
                path,
                enabled,
                _label: _,
            } = root;
            if !enabled {
                return None;
            }

            match AbsolutePathBuf::from_absolute_path_checked(&path) {
                Ok(path) => Some(path),
                Err(err) => {
                    tracing::warn!(
                        provider = %provider.display(),
                        path = %path,
                        "ignoring invalid skill root provider path: {err}"
                    );
                    None
                }
            }
        })
        .collect()
}

fn brief_stderr(stderr: &[u8]) -> String {
    const MAX_STDERR_BYTES: usize = 256;
    let truncated = stderr.get(..MAX_STDERR_BYTES).unwrap_or(stderr);
    let mut value = String::from_utf8_lossy(truncated).into_owned();
    if stderr.len() > MAX_STDERR_BYTES {
        value.push_str("...");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn build_context_serializes_expected_json() {
        let temp_root = tempdir().expect("create temp root");
        let cwd_path = temp_root.path().join("cwd");
        let home_path = temp_root.path().join("home");
        let codex_home_path = home_path.join(".codex");
        fs::create_dir_all(&cwd_path).expect("create cwd");
        fs::create_dir_all(&codex_home_path).expect("create codex home");

        let cwd = AbsolutePathBuf::try_from(cwd_path.clone()).expect("absolute cwd");
        let home_dir = AbsolutePathBuf::try_from(home_path.clone()).expect("absolute home");
        let codex_home =
            AbsolutePathBuf::try_from(codex_home_path.clone()).expect("absolute codex home");

        let context = build_context(&cwd, &home_dir, &codex_home);

        assert_eq!(
            serde_json::to_value(context).expect("serialize provider context"),
            json!({
                "schema_version": 1,
                "cwd": cwd_path.to_string_lossy(),
                "home": home_path.to_string_lossy(),
                "codex_home": codex_home_path.to_string_lossy(),
                "session_source": "tui",
            })
        );
    }

    #[test]
    fn providers_dir_joins_expected_path() {
        let temp_root = tempdir().expect("create temp root");
        let home_dir =
            AbsolutePathBuf::try_from(temp_root.path().join("home")).expect("absolute home path");

        assert_eq!(
            providers_dir(&home_dir).as_path(),
            temp_root
                .path()
                .join("home")
                .join(".agents")
                .join("codex")
                .join("skill-root-providers.d")
                .as_path()
        );
    }

    #[tokio::test]
    async fn missing_provider_dir_returns_empty() {
        let temp_root = tempdir().expect("create temp root");
        let cwd = AbsolutePathBuf::try_from(temp_root.path().join("cwd")).expect("absolute cwd");
        let home_dir =
            AbsolutePathBuf::try_from(temp_root.path().join("home")).expect("absolute home");
        let codex_home =
            AbsolutePathBuf::try_from(temp_root.path().join("codex-home")).expect("absolute home");

        fs::create_dir_all(cwd.as_path()).expect("create cwd");
        fs::create_dir_all(home_dir.as_path()).expect("create home");
        fs::create_dir_all(codex_home.as_path()).expect("create codex home");

        let roots = load_provider_roots(&cwd, Some(&home_dir), Some(&codex_home)).await;

        assert_eq!(roots, Vec::<AbsolutePathBuf>::new());
    }

    #[test]
    fn disabled_roots_are_filtered() {
        let temp_root = tempdir().expect("create temp root");
        let enabled_path = temp_root.path().join("enabled");
        let disabled_path = temp_root.path().join("disabled");
        let stdout = json!({
            "roots": [
                { "path": enabled_path.to_string_lossy(), "enabled": true },
                { "path": disabled_path.to_string_lossy(), "enabled": false }
            ]
        })
        .to_string();

        let roots = parse_output(stdout.as_bytes(), Path::new("provider"));

        assert_eq!(
            roots,
            vec![AbsolutePathBuf::try_from(enabled_path).expect("absolute enabled path")]
        );
    }

    #[test]
    fn relative_paths_are_ignored() {
        let temp_root = tempdir().expect("create temp root");
        let absolute_path = temp_root.path().join("skill-root");
        let stdout = json!({
            "roots": [
                { "path": "relative/path" },
                { "path": absolute_path.to_string_lossy() }
            ]
        })
        .to_string();

        let roots = parse_output(stdout.as_bytes(), Path::new("provider"));

        assert_eq!(
            roots,
            vec![AbsolutePathBuf::try_from(absolute_path).expect("absolute skill root path")]
        );
    }

    #[test]
    fn invalid_json_returns_empty() {
        let roots = parse_output(b"not json", Path::new("provider"));

        assert_eq!(roots, Vec::<AbsolutePathBuf>::new());
    }
}
