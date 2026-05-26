pub(crate) fn archive_read_error_message(path: &str, filename: &str, error: &str) -> String {
    format!(
        "failed to read {path} for SHA-256: {error}. missing native archive {filename}; run `just sync-artifacts` or `cargo xtask sync-artifacts` to populate objs/ from the release-objs artifact set. If you are intentionally rebuilding native archives, copy {filename} into objs/ and refresh crates/ton-objs/artifacts_manifest.toml. {env}=1 is only a temporary local escape hatch for SHA verification and does not replace a missing archive.",
        env = "TON_OBJS_DISABLE_ARCHIVE_SHA_VERIFY"
    )
}
