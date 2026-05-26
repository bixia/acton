#[path = "../build_support.rs"]
mod build_support;

#[test]
fn missing_archive_diagnostic_points_to_artifact_sync() {
    let message = build_support::archive_read_error_message(
        "objs/libemulator.a",
        "libemulator.a",
        "permission denied",
    );

    assert!(message.contains("failed to read objs/libemulator.a for SHA-256: permission denied"));
    assert!(message.contains("missing native archive libemulator.a"));
    assert!(message.contains("just sync-artifacts"));
    assert!(message.contains("cargo xtask sync-artifacts"));
    assert!(message.contains("TON_OBJS_DISABLE_ARCHIVE_SHA_VERIFY=1"));
}
