#[test]
fn with_help() {
    let output = test_bin::get_test_bin!("fit2srt-cli")
        .arg("--help")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"Usage: fit2srt-cli [OPTIONS] [FIT_FILES]..."));
}

#[test]
fn without_option() {
    let output = test_bin::get_test_bin("fit2srt-cli")
        .arg("../assets/garmin_g1.fit")
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.5m\n\n"));
}

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_before() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args(["-a", "03:10:00", "../assets/garmin_g1.fit"])
        .output()
        .expect("Failed to launch fit2srt");
    assert!(output
        .stdout
        .starts_with(b"1\n00:00:00,000 --> 00:00:01,000\n1.7m\n\n"));
}

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_time_slot() {
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args([
            "-a",
            "03:10:00",
            "-b",
            "03:10:05",
            "../assets/garmin_g1.fit",
        ])
        .output()
        .expect("Failed to launch fit2srt");
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        r#"1
00:00:00,000 --> 00:00:01,000
1.7m

2
00:00:01,000 --> 00:00:02,000
2.0m

3
00:00:06,000 --> 00:00:16,000
Summary:
Location: 21.939215641468763, 120.74536473490298
Temperature: 31C
Depth: 4.248m (max: 8.908m)

"#
    );
}

// Run test in UTC timezone and in CI
#[test_with::timezone(0)]
fn with_start_time() {
    // The args only work for UTC timezone
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args(["-a", "03:58:29", "-s", "03:58:29", "../assets/713-2.fit"])
        .output()
        .expect("Failed to launch fit2srt");

    assert!(output.stdout.starts_with(
        b"1\n00:55:40,000 --> 00:55:43,000\n1.3m\n\n2\n00:55:43,000 --> 00:55:44,000\n1.5m\n\n3\n00:55:44,000 --> 00:55:47,000\n1.6m\n\n4\n00:55:47,000 --> 00:55:48,000\n1.9m\n\n"
    ));
}

#[test_with::timezone(0)]
fn concat() {
    // starting time 151534
    let output = test_bin::get_test_bin("fit2srt-cli")
        .args([
            "-a",
            "07:15:34",
            "-n",
            "../assets/131-1.fit",
            "../assets/131-2.fit",
        ])
        .output()
        .expect("Failed to launch fit2srt");

    assert!(output.stdout.ends_with(
        b"381\n00:18:33,000 --> 00:02:27,000\n0.5m\n\n382\n00:18:34,000 --> 00:02:29,000\n0.3m\n\n"
    ));
}
