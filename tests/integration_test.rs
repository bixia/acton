#[cfg(test)]
mod common;
#[cfg(test)]
mod integration;
#[cfg(test)]
mod support;

use acton_config::schema::{
    ACTON_SCHEMA_JSON, LINT_REPORT_SCHEMA_JSON, MUTATION_RULES_SCHEMA_JSON,
};
use common::ActonCommandExt;
use std::{fs, process::Command};

const MANUAL_COMMANDS: &[&str] = &[
    "init",
    "new",
    "build",
    "help",
    "hooks",
    "compile",
    "wrapper",
    "disasm",
    "fmt",
    "retrace",
    "test",
    "check",
    "script",
    "run",
    "verify",
    "library",
    "wallet",
    "rpc",
    "localnet",
    "doc",
    "ls",
    "up",
    "doctor",
    "func2tolk",
    "completions",
];

#[test]
fn test_acton_help_long_flag() {
    snapbox::cmd::Command::acton_ui()
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/acton/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_help_short_flag() {
    snapbox::cmd::Command::acton_ui()
        .arg("-h")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/acton/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_version_short_flags_match_long_flag() {
    let long = Command::new(common::acton_exe())
        .arg("--version")
        .output()
        .expect("failed to run acton --version");
    let short = Command::new(common::acton_exe())
        .arg("-v")
        .output()
        .expect("failed to run acton -v");
    let alternative_short = Command::new(common::acton_exe())
        .arg("-V")
        .output()
        .expect("failed to run acton -V");

    assert!(long.status.success(), "acton --version failed: {long:?}");
    assert!(short.status.success(), "acton -v failed: {short:?}");
    assert!(
        alternative_short.status.success(),
        "acton -V failed: {alternative_short:?}"
    );
    assert_eq!(
        short.stdout, long.stdout,
        "acton -v output differed from --version"
    );
    assert_eq!(
        alternative_short.stdout, long.stdout,
        "acton -V output differed from --version"
    );
    assert!(long.stderr.is_empty(), "acton --version wrote to stderr");
    assert!(short.stderr.is_empty(), "acton -v wrote to stderr");
    assert!(
        alternative_short.stderr.is_empty(),
        "acton -V wrote to stderr"
    );
}

#[test]
fn test_acton_help_without_flag() {
    snapbox::cmd::Command::acton_ui()
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""])
        .stderr_eq(snapbox::file!["snapshots/acton/stderr_no_flag.txt"]);
}

#[test]
fn test_acton_lint_shows_check_replacement() {
    snapbox::cmd::Command::acton_ui()
        .arg("lint")
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""])
        .stderr_eq(snapbox::file!["snapshots/lint/stderr.txt"]);
}

#[test]
fn test_acton_lint_with_args_shows_check_replacement() {
    snapbox::cmd::Command::acton_ui()
        .args(["lint", "counter", "--fix"])
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""])
        .stderr_eq(snapbox::file!["snapshots/lint/stderr_with_args.txt"]);
}

#[test]
fn test_acton_compile_rejects_conflicting_stdout_formats() {
    let assert = snapbox::cmd::Command::acton_ui()
        .args(["compile", "contracts/main.tolk", "--json", "--base64-only"])
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""]);
    assert_stderr_contains_all(&assert, &["cannot be used with", "--json", "--base64-only"]);
}

#[test]
fn test_acton_doctor_ignores_project_toolchain_mismatch() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    write_minimal_acton_toml(
        temp_dir.path(),
        r#"
[toolchain]
acton = "0.0.0-doctor-mismatch-test"
"#,
    );

    snapbox::cmd::Command::acton_ui()
        .arg("doctor")
        .current_dir(temp_dir.path())
        .assert()
        .success()
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_disasm_rejects_boc_file_with_address() {
    let assert = snapbox::cmd::Command::acton_ui()
        .args([
            "disasm",
            "contract.boc",
            "--address",
            "UQA_ftKIJsHEAE_UgtFOUK15hPzycZooFuUr8duyY9T3kwwM",
        ])
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""]);
    assert_stderr_contains_all(&assert, &["cannot be used with", "--address", "BOC_FILE"]);
}

#[test]
fn test_acton_library_publish_rejects_local_and_global_together() {
    let assert = snapbox::cmd::Command::acton_ui()
        .args([
            "library",
            "publish",
            "Math",
            "--local",
            "--global",
            "--duration",
            "1d",
            "--wallet",
            "deployer",
            "--yes",
        ])
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""]);
    assert_stderr_contains_all(&assert, &["cannot be used with", "--local", "--global"]);
}

#[test]
fn test_acton_localnet_airdrop_rejects_non_positive_amount() {
    snapbox::cmd::Command::acton_ui()
        .args([
            "localnet",
            "airdrop",
            "UQA_ftKIJsHEAE_UgtFOUK15hPzycZooFuUr8duyY9T3kwwM",
            "--amount",
            "0",
        ])
        .assert()
        .failure()
        .stdout_eq(snapbox::str![""])
        .stderr_eq(snapbox::file![
            "snapshots/localnet_airdrop_non_positive_amount/stderr.txt"
        ]);
}

#[test]
fn test_acton_build_help() {
    snapbox::cmd::Command::acton_ui()
        .arg("build")
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/build/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_help_build() {
    snapbox::cmd::Command::acton_ui()
        .arg("help")
        .arg("build")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/help_build/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_new_help() {
    snapbox::cmd::Command::acton_ui()
        .arg("new")
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/new/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_help_new() {
    snapbox::cmd::Command::acton_ui()
        .arg("help")
        .arg("new")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/help_new/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_rpc_help() {
    snapbox::cmd::Command::acton_ui()
        .arg("rpc")
        .arg("--help")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/rpc/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_rpc_info_help() {
    snapbox::cmd::Command::acton_ui()
        .args(["rpc", "info", "--help"])
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/rpc_info/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_rpc_trace_help() {
    snapbox::cmd::Command::acton_ui()
        .args(["rpc", "trace", "--help"])
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/rpc_trace/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_retrace_help() {
    snapbox::cmd::Command::acton_ui()
        .args(["retrace", "--help"])
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/retrace/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_reverse_retrace_help() {
    let output = Command::new(common::acton_exe())
        .args(["--color", "never", "reverse", "retrace", "--help"])
        .output()
        .expect("failed to run acton reverse retrace --help");

    assert!(
        output.status.success(),
        "acton reverse retrace --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Replay a transaction and emit state-flow JSON",
        "Usage: acton reverse retrace",
        "--output <OUTPUT>",
        "[aliases: --out]",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse retrace --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_collect_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("collect")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse collect --help");

    assert!(
        output.status.success(),
        "acton reverse collect --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Collect account history into a state-flow corpus",
        "Usage: acton reverse collect",
        "--limit <LIMIT>",
        "--output <OUTPUT>",
        "[aliases: --out]",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse collect --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_infer_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("infer")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse infer --help");

    assert!(
        output.status.success(),
        "acton reverse infer --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Infer opcode and effect schema candidates from a state-flow corpus",
        "Usage: acton reverse infer",
        "--artifact-manifest <ARTIFACTS>",
        "--target-id <TARGET_ID>",
        "--output <OUTPUT>",
        "[aliases: --out]",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse infer --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_replay_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("replay")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse replay --help");

    assert!(
        output.status.success(),
        "acton reverse replay --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Replay or mutate a StateFlowTx or corpus artifact and emit a diff",
        "Usage: acton reverse replay",
        "--artifact-manifest <ARTIFACTS>",
        "--target-id <TARGET_ID>",
        "--tx-index <TX_INDEX>",
        "--tx-hash <TX_HASH>",
        "--flip-body-bit <FLIP_BODY_BIT>",
        "--body-boc64 <BODY_BOC64>",
        "--ignore-chksig",
        "--output <OUTPUT>",
        "[aliases: --out]",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse replay --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_report_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("report")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse report --help");

    assert!(
        output.status.success(),
        "acton reverse report --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Generate a state-flow reverse-engineering report",
        "Usage: acton reverse report",
        "--schema <SCHEMA>",
        "--replay <REPLAY>",
        "--artifact-manifest <ARTIFACTS>",
        "--target-id <TARGET_ID>",
        "--output <OUTPUT>",
        "[aliases: --out]",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse report --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_smoke_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("smoke")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse smoke --help");

    assert!(
        output.status.success(),
        "acton reverse smoke --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Run state-flow smoke targets through collect, infer, replay, and report",
        "Usage: acton reverse smoke",
        "--targets <TARGETS>",
        "--target-id <TARGET_ID>",
        "--out-dir <OUT_DIR>",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse smoke --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_verify_artifacts_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("verify-artifacts")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse verify-artifacts --help");

    assert!(
        output.status.success(),
        "acton reverse verify-artifacts --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Validate a state-flow artifact manifest bundle",
        "Usage: acton reverse verify-artifacts",
        "<ARTIFACTS>",
        "--target-id <TARGET_ID>",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse verify-artifacts --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_reverse_analyze_help() {
    let output = snapbox::cmd::Command::acton_ui()
        .arg("reverse")
        .arg("analyze")
        .arg("--help")
        .output()
        .expect("failed to run acton reverse analyze --help");

    assert!(
        output.status.success(),
        "acton reverse analyze --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
    for expected in [
        "Run collect, infer, replay, and report for one target address",
        "Usage: acton reverse analyze",
        "--net <NET>",
        "--limit <LIMIT>",
        "--replay-tx-index <REPLAY_TX_INDEX>",
        "--replay-tx-hash <REPLAY_TX_HASH>",
        "--flip-body-bit <FLIP_BODY_BIT>",
        "--body-boc64 <BODY_BOC64>",
        "--ignore-chksig",
        "--out-dir <OUT_DIR>",
        "--pretty",
    ] {
        assert!(
            stdout.contains(expected),
            "acton reverse analyze --help did not contain {expected:?}.\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_acton_help_retrace() {
    snapbox::cmd::Command::acton_ui()
        .arg("help")
        .arg("retrace")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/help_retrace/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_acton_help_verify() {
    snapbox::cmd::Command::acton_ui()
        .arg("help")
        .arg("verify")
        .assert()
        .success()
        .stdout_eq(snapbox::file!["snapshots/help_verify/stdout.txt"])
        .stderr_eq(snapbox::str![""]);
}

#[test]
fn test_manual_commands_short_help_points_to_detailed_help() {
    for command in MANUAL_COMMANDS {
        let output = Command::new(common::acton_exe())
            .args(["--color", "never", command, "--help"])
            .output()
            .unwrap_or_else(|err| panic!("failed to run acton {command} --help: {err}"));

        assert!(
            output.status.success(),
            "acton {command} --help failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
        let expected = format!("Run 'acton help {command}' for more detailed information.");
        assert!(
            stdout.contains(&expected),
            "acton {command} --help did not contain detailed help pointer.\nExpected: {expected}\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_manual_commands_detailed_help_is_available() {
    for command in MANUAL_COMMANDS {
        let output = Command::new(common::acton_exe())
            .args(["--color", "never", "help", command])
            .output()
            .unwrap_or_else(|err| panic!("failed to run acton help {command}: {err}"));

        assert!(
            output.status.success(),
            "acton help {command} failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );

        let stdout = common::strip_ansi(&String::from_utf8_lossy(&output.stdout));
        let expected = format!("ACTON-{}(1)", command.to_ascii_uppercase());
        assert!(
            stdout.contains(&expected),
            "acton help {command} did not render the generated manual.\nExpected to find: {expected}\nActual stdout:\n{stdout}",
        );
    }
}

#[test]
fn test_commands_index_links_all_documented_command_pages() {
    #[derive(serde::Deserialize)]
    struct CommandsMeta {
        pages: Vec<String>,
    }

    let meta = fs::read_to_string("docs/content/docs/commands/meta.json")
        .expect("failed to read commands meta.json");
    let meta: CommandsMeta =
        serde_json::from_str(&meta).expect("failed to parse commands meta.json");
    let index = fs::read_to_string("docs/content/docs/commands/overview.mdx")
        .expect("failed to read commands overview.mdx");

    for page in meta.pages {
        if page == "overview" || page.starts_with('!') {
            continue;
        }

        let href = format!("href=\"/docs/commands/{page}\"");
        assert!(
            index.contains(&href),
            "commands index is missing a card for {page} ({href})"
        );
    }
}

#[test]
fn test_acton_meta_get_schema_prints_embedded_schema() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = Command::new(common::acton_exe())
        .args(["meta", "get-schema"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap_or_else(|err| panic!("failed to run acton meta get-schema: {err}"));

    assert!(
        output.status.success(),
        "acton meta get-schema failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert_eq!(String::from_utf8_lossy(&output.stdout), ACTON_SCHEMA_JSON);
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn test_acton_meta_get_schema_prints_mutation_rules_schema() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = Command::new(common::acton_exe())
        .args(["meta", "get-schema", "mutation-rules"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap_or_else(|err| panic!("failed to run acton meta get-schema mutation-rules: {err}"));

    assert!(
        output.status.success(),
        "acton meta get-schema mutation-rules failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        MUTATION_RULES_SCHEMA_JSON
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

fn write_minimal_acton_toml(project_root: &std::path::Path, extra: &str) {
    let content = format!(
        r#"[package]
name = "cli-integration-test"
version = "0.0.0"

{extra}"#
    );
    fs::write(project_root.join("Acton.toml"), content).expect("failed to write Acton.toml");
}

fn assert_stderr_contains_all(assert: &snapbox::cmd::OutputAssert, expected: &[&str]) {
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);

    for expected in expected {
        assert!(
            stderr.contains(expected),
            "stderr did not contain {expected:?}\nstderr:\n{stderr}"
        );
    }
}

#[test]
fn test_acton_meta_get_schema_prints_lint_report_schema() {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = Command::new(common::acton_exe())
        .args(["meta", "get-schema", "lint-report"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap_or_else(|err| panic!("failed to run acton meta get-schema lint-report: {err}"));

    assert!(
        output.status.success(),
        "acton meta get-schema lint-report failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        LINT_REPORT_SCHEMA_JSON
    );
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}
