use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Create a spinner progress indicator
pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

/// Create a progress bar
pub fn create_progress_bar(total: u64, msg: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb.set_message(msg.to_string());
    pb
}

/// Print success message
pub fn print_success(msg: &str) {
    println!("\n{} {}", "✓".green().bold(), msg.green());
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("\n{} {}", "✗".red().bold(), msg.red());
}

/// Print info message
pub fn print_info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print warning message
pub fn print_warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Validate package name (lowercase alphanumeric and hyphens only)
pub fn validate_package_name(name: &str) -> bool {
    let re = regex::Regex::new(r"^[a-z0-9-]+$").unwrap();
    re.is_match(name) && name.len() <= 64
}

/// Validate semantic version format
pub fn validate_version(version: &str) -> bool {
    let re = regex::Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    re.is_match(version) && version.len() <= 16
}

/// Parse package specification (name@version or just name)
pub fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(idx) = spec.find('@') {
        let name = spec[..idx].to_string();
        let version = spec[idx + 1..].to_string();
        (name, Some(version))
    } else {
        (spec.to_string(), None)
    }
}
