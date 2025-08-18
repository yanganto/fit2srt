#[test]
fn with_help() {
    let output = test_bin::get_test_bin("fit2srt-cli")
        .arg("--help")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"Usage: fit2srt [OPTIONS] <FIT_FILE>"));
}

#[test]
fn without_option() {
    let output = test_bin::get_test_bin("fit2srt-cli")
        .arg("asset/garmin_g1.fit")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.5M\n\n"));
}

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_before() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args(["-a", "03:10:00", "asset/garmin_g1.fit"])
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.7M\n\n"));
}

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_time_slot() {
    let output = test_bin::get_test_bin("fit2srt-cli")
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

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_start_time() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args(["-a", "03:58:29", "-s", "03:58:29", "asset/713-2.fit"])
        .output()
        .expect("Failed to launch fit2srt");

    assert!(output.stdout.starts_with(
        b"1\n00:55:40,000 --> 00:55:43,000\n1.3M\n\n2\n00:55:43,000 --> 00:55:44,000\n1.5M\n\n3\n00:55:44,000 --> 00:55:47,000\n1.6M\n\n4\n00:55:47,000 --> 00:55:48,000\n1.9M\n\n"
    ));
}
