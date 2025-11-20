use crate::db::models::Event;

pub fn parse_transaction(
    log: &str,
    signature: &str,
    slot: i64,
    block_time: Option<i64>,
) -> Option<Event> {
    let log_lower = log.to_lowercase();
    // Try to parse different event types
    
    // Pattern 1: PackagePublished or Publish instruction
    if log_lower.contains("packagepublished") || log_lower.contains("instruction: publish") || log_lower.contains("program log: publish") || log_lower.contains("package published:") {
        if let Some((package_name, version)) = extract_package_info(log) {
            tracing::debug!("Parsed PackagePublished: {} v{}", package_name, version.as_ref().unwrap_or(&"unknown".to_string()));
            return Some(Event {
                id: 0,
                event_type: "PackagePublished".to_string(),
                package_name,
                version,
                transaction_signature: signature.to_string(),
                slot,
                block_time: block_time.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()),
            });
        }
    }
    
    // Pattern 2: PackageUpdated or Update instruction
    if log_lower.contains("packageupdated") || log_lower.contains("instruction: update") || log_lower.contains("program log: update") {
        if let Some((package_name, version)) = extract_package_info(log) {
            tracing::debug!("Parsed PackageUpdated: {} v{}", package_name, version.as_ref().unwrap_or(&"unknown".to_string()));
            return Some(Event {
                id: 0,
                event_type: "PackageUpdated".to_string(),
                package_name,
                version,
                transaction_signature: signature.to_string(),
                slot,
                block_time: block_time.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()),
            });
        }
    }
    
    // Pattern 3: PackageDownloaded or Download instruction
    if log_lower.contains("packagedownloaded") || log_lower.contains("instruction: download") || log_lower.contains("program log: download") {
        if let Some((package_name, version)) = extract_package_info(log) {
            tracing::debug!("Parsed PackageDownloaded: {} v{}", package_name, version.as_ref().unwrap_or(&"unknown".to_string()));
            return Some(Event {
                id: 0,
                event_type: "PackageDownloaded".to_string(),
                package_name,
                version,
                transaction_signature: signature.to_string(),
                slot,
                block_time: block_time.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()),
            });
        }
    }
    
    // Pattern 4: Generic package events with event field
    if let Some(event_type) = extract_field(log, "event") {
        if let Some((package_name, version)) = extract_package_info(log) {
            tracing::debug!("Parsed generic event: {} for {} v{}", event_type, package_name, version.as_ref().unwrap_or(&"unknown".to_string()));
            return Some(Event {
                id: 0,
                event_type,
                package_name,
                version,
                transaction_signature: signature.to_string(),
                slot,
                block_time: block_time.map(|ts| chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default()),
            });
        }
    }
    
    None
}

fn extract_package_info(log: &str) -> Option<(String, Option<String>)> {
    // Try to extract "package@version" format first (e.g., "Package published: awesome-math-utils@1.0.0")
    if let Some(at_format) = try_extract_at_format(log) {
        return Some(at_format);
    }
    
    let package_name = extract_field(log, "package")
        .or_else(|| extract_field(log, "name"))
        .or_else(|| extract_field(log, "pkg"))?;
    
    let version = extract_field(log, "version")
        .or_else(|| extract_field(log, "ver"));
    
    Some((package_name, version))
}

fn try_extract_at_format(log: &str) -> Option<(String, Option<String>)> {
    // Look for pattern: "something: package-name@version"
    if let Some(colon_pos) = log.rfind(':') {
        let after_colon = &log[colon_pos + 1..].trim();
        // Remove emoji and other leading characters
        let cleaned = after_colon.trim_start_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_');
        
        if let Some(at_pos) = cleaned.find('@') {
            let package_name = cleaned[..at_pos].trim().to_string();
            let rest = &cleaned[at_pos + 1..];
            
            // Extract version (stop at whitespace or end)
            let version = rest.split_whitespace()
                .next()
                .map(|v| v.to_string());
            
            if !package_name.is_empty() && version.is_some() {
                return Some((package_name, version));
            }
        }
    }
    None
}

fn extract_field(log: &str, field: &str) -> Option<String> {
    // Try multiple extraction patterns
    
    // Pattern 1: JSON format "field":"value" or "field": "value"
    if let Some(value) = try_extract_json_field(log, field) {
        return Some(value);
    }
    
    // Pattern 2: key=value format
    if let Some(value) = try_extract_kv_field(log, field) {
        return Some(value);
    }
    
    // Pattern 3: "field: value" format
    if let Some(value) = try_extract_colon_field(log, field) {
        return Some(value);
    }
    
    // Pattern 4: Extract from structured log format
    if let Some(value) = try_extract_structured_field(log, field) {
        return Some(value);
    }
    
    None
}

fn try_extract_json_field(log: &str, field: &str) -> Option<String> {
    // Try "field":"value"
    if let Some(start) = log.find(&format!("\"{}\":", field)) {
        let after_field = &log[start + field.len() + 3..];
        if let Some(value_start) = after_field.find('"') {
            let value_str = &after_field[value_start + 1..];
            if let Some(value_end) = value_str.find('"') {
                let value = value_str[..value_end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    
    // Try "field": "value" (with space)
    if let Some(start) = log.find(&format!("\"{}\": ", field)) {
        let after_field = &log[start + field.len() + 4..];
        if let Some(value_start) = after_field.find('"') {
            let value_str = &after_field[value_start + 1..];
            if let Some(value_end) = value_str.find('"') {
                let value = value_str[..value_end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    
    None
}

fn try_extract_kv_field(log: &str, field: &str) -> Option<String> {
    if let Some(start) = log.find(&format!("{}=", field)) {
        let after_field = &log[start + field.len() + 1..];
        
        // Handle quoted values
        if after_field.starts_with('"') || after_field.starts_with('\'') {
            let quote = after_field.chars().next()?;
            let value_str = &after_field[1..];
            if let Some(value_end) = value_str.find(quote) {
                let value = value_str[..value_end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        } else {
            // Handle unquoted values
            if let Some(end) = after_field.find(|c: char| c.is_whitespace() || c == ',' || c == '}' || c == ')' || c == ']') {
                let value = after_field[..end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            } else {
                // Value might be at the end of the string
                let value = after_field.trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    
    None
}

fn try_extract_colon_field(log: &str, field: &str) -> Option<String> {
    if let Some(start) = log.find(&format!("{}: ", field)) {
        let after_field = &log[start + field.len() + 2..];
        
        // Handle quoted values
        if after_field.starts_with('"') || after_field.starts_with('\'') {
            let quote = after_field.chars().next()?;
            let value_str = &after_field[1..];
            if let Some(value_end) = value_str.find(quote) {
                let value = value_str[..value_end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        } else {
            if let Some(end) = after_field.find(|c: char| c.is_whitespace() || c == ',') {
                let value = after_field[..end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            } else {
                let value = after_field.trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    
    None
}

fn try_extract_structured_field(log: &str, field: &str) -> Option<String> {
    // Handle Anchor-style logs: "Program log: {field} {value}"
    let patterns = [
        format!("Program log: {} ", field),
        format!("Program data: {} ", field),
    ];
    
    for pattern in &patterns {
        if let Some(start) = log.find(pattern) {
            let after_pattern = &log[start + pattern.len()..];
            if let Some(end) = after_pattern.find(|c: char| c == '\n' || c == '\r') {
                let value = after_pattern[..end].trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            } else {
                let value = after_pattern.trim().to_string();
                if !value.is_empty() {
                    return Some(value);
                }
            }
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_format() {
        let log = r#"Program log: {"package":"test-pkg","version":"1.0.0"}"#;
        assert_eq!(extract_field(log, "package"), Some("test-pkg".to_string()));
        assert_eq!(extract_field(log, "version"), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_kv_format() {
        let log = "Program log: package=test-pkg version=1.0.0";
        assert_eq!(extract_field(log, "package"), Some("test-pkg".to_string()));
        assert_eq!(extract_field(log, "version"), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_extract_colon_format() {
        let log = "Program log: package: test-pkg, version: 1.0.0";
        assert_eq!(extract_field(log, "package"), Some("test-pkg".to_string()));
        assert_eq!(extract_field(log, "version"), Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_publish_event() {
        let log = r#"Program log: PackagePublished {"package":"my-pkg","version":"1.0.0"}"#;
        let event = parse_transaction(log, "sig123", 12345, Some(1699900000));
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event_type, "PackagePublished");
        assert_eq!(event.package_name, "my-pkg");
        assert_eq!(event.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_publish_event_case_insensitive() {
        let log = "program log: packagepublished {\"package\":\"my-pkg\",\"version\":\"1.0.0\"}";
        let event = parse_transaction(log, "sig123", 12345, Some(1699900000));
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.event_type, "PackagePublished");
        assert_eq!(event.package_name, "my-pkg");
        assert_eq!(event.version, Some("1.0.0".to_string()));
    }

    #[test]
    fn test_parse_update_and_download_variants() {
        let log = "Instruction: Update {\"package\":\"foo\",\"version\":\"2.0.0\"}";
        let event = parse_transaction(log, "sigU", 1, None);
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "PackageUpdated");

        let log = "program log: download {\"package\":\"foo\",\"version\":\"2.0.0\"}";
        let event = parse_transaction(log, "sigD", 2, None);
        assert!(event.is_some());
        assert_eq!(event.unwrap().event_type, "PackageDownloaded");
    }

    #[test]
    fn test_parse_publish_missing_version() {
        let log = "Program log: PackagePublished {\"package\":\"nover\"}";
        let event = parse_transaction(log, "sigX", 3, None);
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.package_name, "nover");
        assert_eq!(event.version, None);
    }
}
