use devpanel::core::dry_run;

#[test]
fn active_is_true_without_production_feature() {
    assert!(dry_run::active());
}

#[test]
fn is_production_is_false_without_production_feature() {
    assert!(!dry_run::is_production());
}

#[test]
fn mode_label_dev_without_production_feature() {
    assert_eq!(dry_run::mode_label(), "DEV (dry-run)");
}

#[test]
fn log_does_not_panic() {
    dry_run::log("test log message");
    dry_run::log("");
    dry_run::log("message with special chars: <>&\"'");
}

#[test]
fn log_with_long_message_does_not_panic() {
    let long = "x".repeat(10_000);
    dry_run::log(&long);
}

#[test]
fn preview_command_formats_command_without_running_it() {
    assert_eq!(
        dry_run::preview_command("apt-get", &["install", "-y", "apache2"]),
        "[dry-run] would run: apt-get install -y apache2"
    );
    assert_eq!(
        dry_run::preview_command("systemctl", &[]),
        "[dry-run] would run: systemctl"
    );
}

#[tokio::test]
async fn sudo_validation_auto_passes_with_empty_password_in_dry_run() {
    assert!(dry_run::active());
    assert!(devpanel::infra::sudo_prompt::validate_sudo_password(String::new()).await);
}
