#[test]
fn with_help() {
    let output = test_bin::get_test_bin("fit2srt")
        .arg("--help")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"Usage: fit2srt [OPTIONS] <FIT_FILE>"));
}

#[test]
fn without_option() {
    let output = test_bin::get_test_bin("fit2srt")
        .arg("asset/garmin_g1.fit")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.5M\n\n"));
}

#[test]
fn with_before() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt")
        .args(["-a", "03:10:00", "asset/garmin_g1.fit"])
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.7M\n\n"));
}

#[test]
fn with_time_slot() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt")
        .args(["-a", "03:10:00", "-b", "03:10:05", "asset/garmin_g1.fit"])
        .output()
        .expect("Failed to launch fit2srt");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"1
00:00:00,000 --> 00:00:01,000
1.7M

2
00:00:01,000 --> 00:00:02,000
2.0M

"#
    );
}
