use devpanel::core::db::{defaults, keys, DevPanelDb, UserSettings};

fn db() -> DevPanelDb { DevPanelDb::open_in_memory().expect("in-memory db must open") }

#[test] fn get_absent_key_returns_none() { let d = db(); assert!(d.get("nonexistent.key").unwrap().is_none()); }
#[test] fn set_then_get_returns_value() { let d = db(); d.set("test.key", "hello").unwrap(); assert_eq!(d.get("test.key").unwrap().as_deref(), Some("hello")); }
#[test] fn set_overwrites_existing_value() { let d = db(); d.set("k", "first").unwrap(); d.set("k", "second").unwrap(); assert_eq!(d.get("k").unwrap().as_deref(), Some("second")); }
#[test] fn delete_removes_key() { let d = db(); d.set("k", "v").unwrap(); d.delete("k").unwrap(); assert!(d.get("k").unwrap().is_none()); }
#[test] fn delete_nonexistent_is_ok() { let d = db(); d.delete("does.not.exist").unwrap(); }
#[test] fn get_or_returns_value_when_present() { let d = db(); d.set("level", "error").unwrap(); assert_eq!(d.get_or("level", "warn"), "error"); }
#[test] fn get_or_returns_default_when_absent() { let d = db(); assert_eq!(d.get_or("absent", "my_default"), "my_default"); }
#[test] fn set_bool_true_stores_true_string() { let d = db(); d.set_bool("flag", true).unwrap(); assert_eq!(d.get("flag").unwrap().as_deref(), Some("true")); }
#[test] fn set_bool_false_stores_false_string() { let d = db(); d.set_bool("flag", false).unwrap(); assert_eq!(d.get("flag").unwrap().as_deref(), Some("false")); }
#[test] fn get_bool_true_when_stored_true() { let d = db(); d.set("flag", "true").unwrap(); assert!(d.get_bool("flag", false)); }
#[test] fn get_bool_false_when_stored_false() { let d = db(); d.set("flag", "false").unwrap(); assert!(!d.get_bool("flag", true)); }
#[test] fn get_bool_uses_default_when_absent() { let d = db(); assert!(d.get_bool("absent", true)); assert!(!d.get_bool("absent", false)); }
#[test] fn set_u32_then_get_u32() { let d = db(); d.set_u32("toast_ms", 5000).unwrap(); assert_eq!(d.get_u32("toast_ms", 4000), 5000); }
#[test] fn get_u32_uses_default_when_absent() { let d = db(); assert_eq!(d.get_u32("absent", 1234), 1234); }
#[test] fn get_u32_uses_default_on_corrupt_value() { let d = db(); d.set("toast_ms", "not-a-number").unwrap(); assert_eq!(d.get_u32("toast_ms", 4000), 4000); }
#[test] fn all_settings_empty_on_fresh_db() { let d = db(); assert!(d.all_settings().unwrap().is_empty()); }
#[test] fn all_settings_returns_all_inserted_pairs() { let d = db(); d.set("a","1").unwrap(); d.set("b","2").unwrap(); d.set("c","3").unwrap(); let pairs = d.all_settings().unwrap(); assert_eq!(pairs.len(), 3); assert_eq!(pairs[0].0, "a"); }
#[test] fn vhost_meta_absent_returns_none() { let d = db(); assert!(d.get_vhost_meta("nonexistent.local").unwrap().is_none()); }
#[test] fn set_vhost_meta_then_get() { let d = db(); d.set_vhost_meta("myapp.local", "production", "Main site").unwrap(); let meta = d.get_vhost_meta("myapp.local").unwrap().expect("must be Some"); assert_eq!(meta.0, "production"); assert_eq!(meta.1, "Main site"); }
#[test] fn set_vhost_meta_overwrites() { let d = db(); d.set_vhost_meta("app.local", "dev", "old").unwrap(); d.set_vhost_meta("app.local", "staging", "new").unwrap(); let meta = d.get_vhost_meta("app.local").unwrap().expect("must be Some"); assert_eq!(meta.0, "staging"); }
#[test] fn all_vhost_meta_returns_all_rows() { let d = db(); d.set_vhost_meta("a.local","dev","").unwrap(); d.set_vhost_meta("b.local","prod","live").unwrap(); let all = d.all_vhost_meta().unwrap(); assert_eq!(all.len(), 2); assert_eq!(all[0].0, "a.local"); }
#[test] fn user_settings_load_uses_defaults_on_fresh_db() { let d = db(); let us = UserSettings::load(&d); assert_eq!(us.apache_log_level, defaults::APACHE_LOG_LEVEL); assert!(us.apache_auto_reload); assert!(us.ui_confirm_deletes); assert_eq!(us.ui_toast_duration_ms, 4000); assert_eq!(us.ssh_default_key_type, defaults::SSH_DEFAULT_KEY_TYPE); }
#[test] fn user_settings_default_matches_db_defaults() {
    let us = UserSettings::default();
    assert_eq!(us.apache_log_level, defaults::APACHE_LOG_LEVEL);
    assert_eq!(us.ui_toast_duration_ms, 4000);
    assert_eq!(us.ssh_default_key_type, defaults::SSH_DEFAULT_KEY_TYPE);
}
#[test] fn user_settings_save_then_load_is_identity() {
    let d = db();
    let original = UserSettings { apache_log_level: "error".into(), apache_auto_reload: false,
        php_default_version: "8.2".into(), php_display_errors: false,
        projects_open_command: "code".into(), ui_confirm_deletes: false,
        ui_toast_duration_ms: 2000, ui_show_setup_log: false,
        ssh_default_key_type: "RSA 4096".into(), editor_command: "vim".into() };
    original.save(&d).unwrap();
    let reloaded = UserSettings::load(&d);
    assert_eq!(reloaded.apache_log_level, original.apache_log_level);
    assert_eq!(reloaded.apache_auto_reload, original.apache_auto_reload);
    assert_eq!(reloaded.ui_toast_duration_ms, original.ui_toast_duration_ms);
    assert_eq!(reloaded.editor_command, original.editor_command);
}
#[test] fn user_settings_partial_db_uses_defaults_for_missing() { let d = db(); d.set(keys::APACHE_LOG_LEVEL, "debug").unwrap(); let us = UserSettings::load(&d); assert_eq!(us.apache_log_level, "debug"); assert_eq!(us.ui_toast_duration_ms, 4000); }
