use devpanel::core::app_config;

#[test]
fn app_config_loads_window_metrics() {
    let metrics = app_config::window_metrics();

    assert!(metrics.width >= 320.0);
    assert!(metrics.height >= 320.0);
    assert!(metrics.min_width >= 320.0);
    assert!(metrics.min_height >= 320.0);
}

#[test]
fn app_config_loads_ui_metrics() {
    let text = app_config::text_metrics();
    let icons = app_config::icon_metrics();
    let controls = app_config::control_metrics();
    let panels = app_config::panel_metrics();

    assert!((8..=72).contains(&text.title));
    assert!((8..=72).contains(&text.modal_title));
    assert!((8..=72).contains(&text.dialog_title));
    assert!((8..=72).contains(&text.body));
    assert!(icons.sidebar_logo >= 8.0);
    assert!(controls.button_height >= 20.0);
    assert!((8..=40).contains(&controls.checkbox_size));
    assert!((8..=40).contains(&controls.large_checkbox_size));
    assert!(controls.modal_log_height >= 80.0);
    assert!(controls.form_dropdown_width >= 80.0);
    assert!(panels.notification_width >= 180.0);
    assert!(panels.sudo_dialog_width >= 280.0);
    assert!(panels.installer_log_height >= 80.0);
    assert!(panels.ssh_keys_list_height >= 120.0);
    assert!(panels.tools_list_height >= 120.0);
    assert!(panels.tools_log_height >= 80.0);
    assert!(panels.tools_compact_log_height >= 80.0);
}

#[test]
fn ui_config_draft_validates_current_values() {
    let draft = app_config::UiConfigDraft::current();

    assert!(draft.validate().is_ok());
}

#[test]
fn ui_config_draft_rejects_invalid_numbers() {
    let mut draft = app_config::UiConfigDraft::current();
    draft.window_width = "wide".to_string();

    assert!(draft.validate().is_err());
}

#[test]
fn app_config_loads_php_versions_and_latest_branch() {
    assert_eq!(app_config::latest_php_version(), "8.5");
    assert_eq!(
        app_config::php_version_numbers(),
        vec![
            "5.6", "7.0", "7.1", "7.2", "7.3", "7.4", "8.0", "8.1", "8.2", "8.3", "8.4", "8.5",
        ]
    );
}
