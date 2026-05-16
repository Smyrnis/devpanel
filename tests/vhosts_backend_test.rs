use devpanel::tabs::vhosts::VHostEntry;
use devpanel::tabs::vhosts::backend::{build_conf_content, parse_vhosts_from_content};

fn entry(server_name: &str, document_root: &str, php: Option<&str>, idx: usize) -> VHostEntry {
    VHostEntry {
        server_name: server_name.into(),
        document_root: document_root.into(),
        php_version: php.map(|s| s.to_string()),
        https_enabled: false,
        tag: String::new(),
        index: idx,
    }
}

#[test]
fn parse_empty_string_returns_empty() {
    assert!(parse_vhosts_from_content("").is_empty());
}
#[test]
fn parse_comment_only_returns_empty() {
    assert!(parse_vhosts_from_content("# DevPanel managed VirtualHosts\n").is_empty());
}
#[test]
fn parse_single_vhost_no_php() {
    let content = "<VirtualHost *:80>\n    ServerName myproject.local\n    DocumentRoot /home/user/projects/myproject/public\n</VirtualHost>\n";
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].server_name, "myproject.local");
    assert_eq!(
        entries[0].document_root,
        "/home/user/projects/myproject/public"
    );
    assert!(entries[0].php_version.is_none());
    assert!(!entries[0].https_enabled);
    assert_eq!(entries[0].index, 0);
}
#[test]
fn parse_single_vhost_with_php_pinned() {
    let content = "<VirtualHost *:80>\n    ServerName shop.local\n    DocumentRoot /var/www/shop\n    <Directory /var/www/shop>\n        SetHandler application/x-httpd-php8.2\n    </Directory>\n</VirtualHost>\n";
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].php_version.as_deref(), Some("8.2"));
}
#[test]
fn parse_multiple_vhosts_assigns_sequential_indexes() {
    let content = "<VirtualHost *:80>\n    ServerName alpha.local\n    DocumentRoot /srv/alpha\n</VirtualHost>\n<VirtualHost *:80>\n    ServerName beta.local\n    DocumentRoot /srv/beta\n</VirtualHost>\n";
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].index, 0);
    assert_eq!(entries[1].index, 1);
}
#[test]
fn parse_vhost_missing_servername_is_skipped() {
    let content = "<VirtualHost *:80>\n    DocumentRoot /var/www/html\n</VirtualHost>\n";
    assert!(parse_vhosts_from_content(content).is_empty());
}
#[test]
fn parse_vhost_case_insensitive_directives() {
    let content = "<virtualhost *:80>\n    servername case.local\n    documentroot /var/www/case\n</virtualhost>\n";
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].server_name, "case.local");
}
#[test]
fn build_empty_entries_produces_header_only() {
    let out = build_conf_content(&[]);
    assert!(out.contains("DevPanel managed VirtualHosts"));
    assert!(!out.contains("<VirtualHost"));
}
#[test]
fn build_single_entry_no_php() {
    let entries = vec![entry(
        "myapp.local",
        "/home/user/projects/myapp/public",
        None,
        0,
    )];
    let out = build_conf_content(&entries);
    assert!(out.contains("ServerName myapp.local"));
    assert!(out.contains("ServerAlias www.myapp.local"));
    assert!(!out.contains("SetHandler"));
}
#[test]
fn build_single_entry_with_php_produces_sethandler() {
    let entries = vec![entry("shop.local", "/var/www/shop", Some("8.2"), 0)];
    let out = build_conf_content(&entries);
    assert!(out.contains("SetHandler application/x-httpd-php8.2"));
}
#[test]
fn build_https_entry_produces_port_443_block() {
    let mut e = entry("secure.local", "/var/www/secure", Some("8.2"), 0);
    e.https_enabled = true;
    let out = build_conf_content(&[e]);
    assert!(out.contains("<VirtualHost *:443>"));
    assert!(out.contains("SSLEngine on"));
    assert!(out.contains("secure_local.pem"));
}
#[test]
fn parse_https_entry_sets_flag_once() {
    let content = "<VirtualHost *:80>\nServerName secure.local\nDocumentRoot /srv/secure\n</VirtualHost>\n<VirtualHost *:443>\nServerName secure.local\nDocumentRoot /srv/secure\nSSLEngine on\n</VirtualHost>\n";
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert!(entries[0].https_enabled);
}
#[test]
fn build_uses_dot_to_underscore_slug_for_log_paths() {
    let entries = vec![entry("my.project.local", "/srv/mp", None, 0)];
    let out = build_conf_content(&entries);
    assert!(out.contains("my_project_local_error.log"));
    assert!(out.contains("my_project_local_access.log"));
}
#[test]
fn round_trip_parse_build_parse_is_stable() {
    let original = vec![
        entry("app.local", "/var/www/app", Some("8.2"), 0),
        entry("blog.local", "/var/www/blog", None, 1),
    ];
    let conf_text = build_conf_content(&original);
    let reparsed = parse_vhosts_from_content(&conf_text);
    assert_eq!(reparsed.len(), original.len());
    for (orig, rep) in original.iter().zip(reparsed.iter()) {
        assert_eq!(orig.server_name, rep.server_name);
        assert_eq!(orig.document_root, rep.document_root);
        assert_eq!(orig.php_version, rep.php_version);
    }
}
#[test]
fn round_trip_trailing_slash_stripped_from_server_name() {
    let entries = vec![entry("slash.local/", "/srv/slash", None, 0)];
    let conf_text = build_conf_content(&entries);
    let reparsed = parse_vhosts_from_content(&conf_text);
    assert_eq!(reparsed.len(), 1);
    assert_eq!(reparsed[0].server_name, "slash.local");
}
#[test]
fn vhost_entry_is_partial_eq() {
    let a = entry("a.local", "/srv/a", Some("8.2"), 0);
    let b = entry("a.local", "/srv/a", Some("8.2"), 0);
    assert_eq!(a, b);
}
