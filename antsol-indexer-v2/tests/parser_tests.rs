use antsol_indexer_v2::indexer::parser::parse_transaction;

#[test]
fn test_parse_json_format_publish() {
    let log = r#"Program log: PackagePublished {"package":"test-pkg","version":"1.0.0"}"#;
    let event = parse_transaction(log, "sig123abc", 12345, Some(1699900000));
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.event_type, "PackagePublished");
    assert_eq!(event.package_name, "test-pkg");
    assert_eq!(event.version, Some("1.0.0".to_string()));
    assert_eq!(event.transaction_signature, "sig123abc");
    assert_eq!(event.slot, 12345);
}

#[test]
fn test_parse_kv_format_update() {
    let log = "Program log: Instruction: Update package=my-package version=2.0.0";
    let event = parse_transaction(log, "sig456def", 67890, Some(1699900000));
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.event_type, "PackageUpdated");
    assert_eq!(event.package_name, "my-package");
    assert_eq!(event.version, Some("2.0.0".to_string()));
}

#[test]
fn test_parse_colon_format_download() {
    let log = "Program log: Download package: awesome-lib, version: 3.5.1";
    let event = parse_transaction(log, "sig789ghi", 11111, Some(1699900000));
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.event_type, "PackageDownloaded");
    assert_eq!(event.package_name, "awesome-lib");
    assert_eq!(event.version, Some("3.5.1".to_string()));
}

#[test]
fn test_parse_no_version() {
    let log = r#"Program log: PackagePublished {"package":"no-version-pkg"}"#;
    let event = parse_transaction(log, "sigXYZ", 22222, None);
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.package_name, "no-version-pkg");
    assert_eq!(event.version, None);
}

#[test]
fn test_parse_invalid_log() {
    let log = "Program log: Some random log message";
    let event = parse_transaction(log, "sigABC", 33333, None);
    
    assert!(event.is_none());
}

#[test]
fn test_parse_multiple_formats() {
    let logs = vec![
        (r#"Program log: PackagePublished {"package":"json-pkg","version":"1.0.0"}"#, "json-pkg", "1.0.0"),
        ("Program log: Instruction: Publish package=kv-pkg version=2.0.0", "kv-pkg", "2.0.0"),
        ("Program log: Publish package: colon-pkg, version: 3.0.0", "colon-pkg", "3.0.0"),
    ];
    
    for (i, (log, expected_name, expected_version)) in logs.iter().enumerate() {
        let event = parse_transaction(log, &format!("sig{}", i), i as i64, None);
        assert!(event.is_some(), "Failed to parse: {}", log);
        let event = event.unwrap();
        assert_eq!(event.package_name, *expected_name);
        assert_eq!(event.version, Some(expected_version.to_string()));
    }
}

#[test]
fn test_parse_quoted_values() {
    let log = r#"Program log: Update package="quoted-pkg" version="1.2.3""#;
    let event = parse_transaction(log, "sig999", 44444, None);
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.package_name, "quoted-pkg");
    assert_eq!(event.version, Some("1.2.3".to_string()));
}

#[test]
fn test_parse_edge_cases() {
    // Package name with special characters
    let log = r#"Program log: PackagePublished {"package":"@scope/my-pkg","version":"1.0.0-beta.1"}"#;
    let event = parse_transaction(log, "sigEDGE", 55555, None);
    
    assert!(event.is_some());
    let event = event.unwrap();
    assert_eq!(event.package_name, "@scope/my-pkg");
    assert_eq!(event.version, Some("1.0.0-beta.1".to_string()));
}
