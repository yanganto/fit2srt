#[test]
fn with_help() {
    let output = test_bin::get_test_bin("fit2srt")
    .arg("--help")
    .output()
    .expect("Failed to launch fit2srt");
    assert!(
        output.stdout.starts_with(b"Usage: fit2srt [OPTIONS] <FIT_FILE>")
    );
}
