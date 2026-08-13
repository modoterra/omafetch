use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn install_sh() -> PathBuf {
    repo_root().join("install.sh")
}

fn bash_output(args: &[&str]) -> Output {
    Command::new("bash")
        .args(args)
        .output()
        .expect("failed to spawn bash")
}

fn sourced(body: &str) -> Output {
    Command::new("bash")
        .args([
            "-c",
            &format!("set -euo pipefail; source \"$INSTALL_SH\"; {body}"),
        ])
        .env("INSTALL_SH", install_sh())
        .output()
        .expect("failed to spawn bash")
}

fn assert_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout_text(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

#[test]
fn install_script_is_valid_bash() {
    let output = bash_output(&["-n", install_sh().to_str().expect("utf-8 path")]);
    assert_success(&output, "bash -n install.sh");
}

#[test]
fn help_exits_zero_without_network() {
    let output = bash_output(&[install_sh().to_str().expect("utf-8 path"), "--help"]);
    assert_success(&output, "install.sh --help");
    let stdout = stdout_text(&output);
    assert!(stdout.contains("--prefix"), "help should describe --prefix");
    assert!(
        stdout.contains("--version"),
        "help should describe --version"
    );
}

#[test]
fn help_works_when_piped_to_bash() {
    let script = std::fs::read_to_string(install_sh()).expect("read install.sh");
    let output = Command::new("bash")
        .args(["-s", "--", "--help"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(script.as_bytes())?;
            child.wait_with_output()
        })
        .expect("pipe install.sh to bash");
    assert_success(&output, "cat install.sh | bash -s -- --help");
    assert!(
        stdout_text(&output).contains("--prefix"),
        "piped help should describe --prefix"
    );
}

#[test]
fn unknown_option_fails() {
    let output = bash_output(&[install_sh().to_str().expect("utf-8 path"), "--nope"]);
    assert!(!output.status.success(), "unknown option should fail");
}

#[test]
fn normalizes_release_versions() {
    let output = sourced(r#"printf '%s\n' "$(normalize_version v0.1.0)""#);
    assert_success(&output, "normalize_version v0.1.0");
    assert_eq!(stdout_text(&output), "0.1.0");

    let output = sourced(r#"printf '%s\n' "$(normalize_version 1.2.3)""#);
    assert_success(&output, "normalize_version 1.2.3");
    assert_eq!(stdout_text(&output), "1.2.3");
}

#[test]
fn rejects_invalid_versions() {
    let output = sourced(r#"normalize_version '../evil'"#);
    assert!(!output.status.success(), "path-like version should fail");

    let output = sourced(r#"normalize_version latest"#);
    assert!(
        !output.status.success(),
        "latest must be resolved before normalize_version"
    );
}

#[test]
fn maps_supported_linux_targets() {
    let output = sourced(r#"printf '%s\n' "$(artifact_target_from Linux x86_64)""#);
    assert_success(&output, "artifact_target_from Linux x86_64");
    assert_eq!(stdout_text(&output), "x86_64-unknown-linux-gnu");

    let output = sourced(r#"printf '%s\n' "$(artifact_target_from Linux amd64)""#);
    assert_success(&output, "artifact_target_from Linux amd64");
    assert_eq!(stdout_text(&output), "x86_64-unknown-linux-gnu");
}

#[test]
fn rejects_unsupported_platforms() {
    let output = sourced(r#"artifact_target_from Darwin x86_64"#);
    assert!(!output.status.success(), "macOS should be rejected");

    let output = sourced(r#"artifact_target_from Linux aarch64"#);
    assert!(!output.status.success(), "aarch64 should be rejected");
}

#[test]
fn builds_release_artifact_names() {
    let output = sourced(r#"printf '%s\n' "$(artifact_basename 0.1.0 x86_64-unknown-linux-gnu)""#);
    assert_success(&output, "artifact_basename");
    assert_eq!(
        stdout_text(&output),
        "omafetch-0.1.0-x86_64-unknown-linux-gnu"
    );
}

#[test]
fn builds_destination_paths() {
    let output = sourced(r#"printf '%s\n' "$(destination_path /usr/local/)""#);
    assert_success(&output, "destination_path");
    assert_eq!(stdout_text(&output), "/usr/local/bin/omafetch");
}

#[test]
fn builds_about_launcher_path() {
    let output = sourced(r#"printf '%s\n' "$(about_launcher_path /usr/local/)""#);
    assert_success(&output, "about_launcher_path");
    assert_eq!(stdout_text(&output), "/usr/local/bin/omarchy-launch-about");
}

#[test]
fn parse_args_sets_defaults_and_flags() {
    let output = sourced(
        r#"
        unset PREFIX VERSION ACTION
        HOME=/tmp/omafetch-home
        parse_args
        printf '%s %s %s\n' "$ACTION" "$PREFIX" "$VERSION"
        "#,
    );
    assert_success(&output, "parse_args defaults");
    assert_eq!(
        stdout_text(&output),
        "install /tmp/omafetch-home/.local latest"
    );

    let output = sourced(
        r#"
        parse_args --prefix /usr/local --version v0.1.0
        printf '%s %s %s\n' "$ACTION" "$PREFIX" "$VERSION"
        "#,
    );
    assert_success(&output, "parse_args flags");
    assert_eq!(stdout_text(&output), "install /usr/local v0.1.0");

    let output = sourced(
        r#"
        parse_args --uninstall --prefix /opt
        printf '%s %s\n' "$ACTION" "$PREFIX"
        "#,
    );
    assert_success(&output, "parse_args uninstall");
    assert_eq!(stdout_text(&output), "uninstall /opt");
}

#[test]
fn cleanup_temp_dir_works_after_function_return() {
    let output = sourced(
        r#"
        work="$(mktemp -d)"
        register_from_function() {
          local unused
          unused="$1"
          register_temp_dir "$unused"
        }
        register_from_function "$work"
        [[ -d "$work" ]]
        cleanup_temp_dir
        [[ ! -d "$work" ]]
        printf 'ok\n'
        "#,
    );
    assert_success(&output, "cleanup_temp_dir after local scope ends");
    assert_eq!(stdout_text(&output), "ok");
}

#[test]
fn verifies_release_checksum_files() {
    let output = sourced(
        r#"
        work="$(mktemp -d)"
        trap 'rm -rf "$work"' EXIT
        printf 'omafetch\n' > "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
        (
            cd "$work"
            sha256sum "omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz" \
                > "omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        )
        verify_checksum \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz" \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        printf 'ok\n'
        "#,
    );
    assert_success(&output, "verify_checksum matching");
    assert_eq!(stdout_text(&output), "ok");

    let output = sourced(
        r#"
        work="$(mktemp -d)"
        trap 'rm -rf "$work"' EXIT
        printf 'omafetch\n' > "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
        hash="$(sha256sum "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz" | awk '{ print $1 }')"
        printf '%s  dist/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz\n' "$hash" \
            > "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        verify_checksum \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz" \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        printf 'ok\n'
        "#,
    );
    assert_success(&output, "verify_checksum with release path prefix");
    assert_eq!(stdout_text(&output), "ok");

    let output = sourced(
        r#"
        work="$(mktemp -d)"
        trap 'rm -rf "$work"' EXIT
        printf 'omafetch\n' > "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz"
        printf '0000000000000000000000000000000000000000000000000000000000000000  omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz\n' \
            > "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        verify_checksum \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz" \
            "$work/omafetch-0.1.0-x86_64-unknown-linux-gnu.tar.gz.sha256"
        "#,
    );
    assert!(!output.status.success(), "mismatched checksum should fail");
}
