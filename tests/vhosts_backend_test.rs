use devpanel::tabs::vhosts::backend::{build_conf_content, parse_vhosts_from_content};
use devpanel::tabs::vhosts::VHostEntry;

fn entry(server_name: &str, document_root: &str, php: Option<&str>, idx: usize) -> VHostEntry {
    VHostEntry {
        server_name:   server_name.into(),
        document_root: document_root.into(),
        php_version:   php.map(|s| s.to_string()),
        index:         idx,
    }
}

#[test]
fn parse_empty_string_returns_empty() {
    assert!(parse_vhosts_from_content("").is_empty());
}

#[test]
fn parse_comment_only_returns_empty() {
    let content = "# DevPanel managed VirtualHosts\n# Managed by DevPanel\n";
    assert!(parse_vhosts_from_content(content).is_empty());
}

#[test]
fn parse_single_vhost_no_php() {
    let content = r#"
<VirtualHost *:80>
    ServerName myproject.local
    DocumentRoot /home/user/projects/myproject/public
</VirtualHost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].server_name, "myproject.local");
    assert_eq!(entries[0].document_root, "/home/user/projects/myproject/public");
    assert!(entries[0].php_version.is_none());
    assert_eq!(entries[0].index, 0);
}

#[test]
fn parse_single_vhost_with_php_pinned() {
    let content = r#"
<VirtualHost *:80>
    ServerName shop.local
    DocumentRoot /var/www/shop
    <Directory /var/www/shop>
        SetHandler application/x-httpd-php8.2
    </Directory>
</VirtualHost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].server_name, "shop.local");
    assert_eq!(entries[0].php_version.as_deref(), Some("8.2"));
}

#[test]
fn parse_multiple_vhosts_assigns_sequential_indexes() {
    let content = r#"
<VirtualHost *:80>
    ServerName alpha.local
    DocumentRoot /srv/alpha
</VirtualHost>
<VirtualHost *:80>
    ServerName beta.local
    DocumentRoot /srv/beta
</VirtualHost>
<VirtualHost *:80>
    ServerName gamma.local
    DocumentRoot /srv/gamma
</VirtualHost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].server_name, "alpha.local");
    assert_eq!(entries[0].index, 0);
    assert_eq!(entries[1].server_name, "beta.local");
    assert_eq!(entries[1].index, 1);
    assert_eq!(entries[2].server_name, "gamma.local");
    assert_eq!(entries[2].index, 2);
}

#[test]
fn parse_vhost_missing_servername_is_skipped() {
    let content = r#"
<VirtualHost *:80>
    DocumentRoot /var/www/html
</VirtualHost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert!(entries.is_empty(), "entry without ServerName must be skipped");
}

#[test]
fn parse_vhost_case_insensitive_directives() {
    let content = r#"
<virtualhost *:80>
    servername case.local
    documentroot /var/www/case
</virtualhost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].server_name, "case.local");
}

#[test]
fn parse_mixed_php_versions_in_multiple_vhosts() {
    let content = r#"
<VirtualHost *:80>
    ServerName old.local
    DocumentRoot /srv/old
    <Directory /srv/old>
        SetHandler application/x-httpd-php7.4
    </Directory>
</VirtualHost>
<VirtualHost *:80>
    ServerName new.local
    DocumentRoot /srv/new
    <Directory /srv/new>
        SetHandler application/x-httpd-php8.3
    </Directory>
</VirtualHost>
<VirtualHost *:80>
    ServerName global.local
    DocumentRoot /srv/global
</VirtualHost>
"#;
    let entries = parse_vhosts_from_content(content);
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].php_version.as_deref(), Some("7.4"));
    assert_eq!(entries[1].php_version.as_deref(), Some("8.3"));
    assert!(entries[2].php_version.is_none());
}

#[test]
fn build_empty_entries_produces_header_only() {
    let out = build_conf_content(&[]);
    assert!(out.contains("DevPanel managed VirtualHosts"));
    assert!(!out.contains("<VirtualHost"));
}

#[test]
fn build_single_entry_no_php() {
    let entries = vec![entry("myapp.local", "/home/user/projects/myapp/public", None, 0)];
    let out = build_conf_content(&entries);
    assert!(out.contains("ServerName myapp.local"));
    assert!(out.contains("ServerAlias www.myapp.local"));
    assert!(out.contains("DocumentRoot /home/user/projects/myapp/public"));
    assert!(!out.contains("SetHandler"));
}

#[test]
fn build_single_entry_with_php_produces_sethandler() {
    let entries = vec![entry("shop.local", "/var/www/shop", Some("8.2"), 0)];
    let out = build_conf_content(&entries);
    assert!(out.contains("SetHandler application/x-httpd-php8.2"));
}

#[test]
fn build_uses_dot_to_underscore_slug_for_log_paths() {
    let entries = vec![entry("my.project.local", "/srv/mp", None, 0)];
    let out = build_conf_content(&entries);
    // Dots in ServerName must be replaced with underscores in log filenames.
    assert!(out.contains("my_project_local_error.log"));
    assert!(out.contains("my_project_local_access.log"));
}

#[test]
fn build_multiple_entries_all_present() {
    let entries = vec![
        entry("alpha.local", "/srv/alpha", None, 0),
        entry("beta.local",  "/srv/beta",  Some("8.1"), 1),
    ];
    let out = build_conf_content(&entries);
    assert!(out.contains("ServerName alpha.local"));
    assert!(out.contains("ServerName beta.local"));
    assert!(out.contains("SetHandler application/x-httpd-php8.1"));
    // alpha should NOT have a SetHandler line
    let alpha_block_start = out.find("ServerName alpha.local").unwrap();
    let beta_block_start  = out.find("ServerName beta.local").unwrap();
    let alpha_block       = &out[alpha_block_start..beta_block_start];
    assert!(!alpha_block.contains("SetHandler"));
}

#[test]
fn round_trip_parse_build_parse_is_stable() {
    let original = vec![
        entry("app.local",   "/var/www/app",   Some("8.2"), 0),
        entry("blog.local",  "/var/www/blog",  None,        1),
        entry("api.local",   "/var/www/api",   Some("8.1"), 2),
    ];
    let conf_text  = build_conf_content(&original);
    let reparsed   = parse_vhosts_from_content(&conf_text);

    assert_eq!(reparsed.len(), original.len());
    for (orig, rep) in original.iter().zip(reparsed.iter()) {
        assert_eq!(orig.server_name,   rep.server_name,   "ServerName mismatch");
        assert_eq!(orig.document_root, rep.document_root, "DocumentRoot mismatch");
        assert_eq!(orig.php_version,   rep.php_version,   "PHP version mismatch");
    }
}

#[test]
fn round_trip_trailing_slash_stripped_from_server_name() {
    // build_conf_content strips trailing slash from server names.
    let entries = vec![entry("slash.local/", "/srv/slash", None, 0)];
    let conf_text = build_conf_content(&entries);
    let reparsed  = parse_vhosts_from_content(&conf_text);
    assert_eq!(reparsed.len(), 1);
    assert_eq!(reparsed[0].server_name, "slash.local");
}
