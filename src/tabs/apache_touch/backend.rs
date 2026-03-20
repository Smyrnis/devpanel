// src/tabs/apache_touch/backend.rs — VirtualHost setup: directory checks, hosts, Apache config
#![allow(dead_code, unused)]

use super::LogEntry;

/// Run the full VirtualHost setup pipeline.
/// Returns a list of log entries and a final success flag.
pub async fn run_setup(
    project_name:   String,
    base_dir:       String,
    apache_conf:    String,
    auth_json_path: String,
    password:       String,
) -> (Vec<LogEntry>, bool) {
    let mut log  = Vec::new();
    let mut ok   = true;

    let project_dir = std::path::PathBuf::from(&base_dir).join(&project_name);
    let local_host  = format!("{}.local", project_name);
    let doc_root    = project_dir.join("public");

    // 1. Check the project directory
    log.push(LogEntry::cmd(format!("Checking {}", project_dir.display())));
    if !project_dir.exists() {
        log.push(LogEntry::err(format!("Directory {} not found", project_dir.display())));
        return (log, false);
    }
    log.push(LogEntry::ok(format!("Found {}", project_dir.display())));

    // 2. .env setup
    let env_example = project_dir.join(".env.example");
    let env_file    = project_dir.join(".env");
    if env_example.exists() && !env_file.exists() {
        log.push(LogEntry::cmd("Copying .env.example → .env"));
        match tokio::fs::copy(&env_example, &env_file).await {
            Ok(_)  => log.push(LogEntry::ok(".env created")),
            Err(e) => { log.push(LogEntry::warn(format!(".env copy failed: {}", e))); }
        }
    }

    // 3. auth.json
    if !auth_json_path.is_empty() {
        let src = std::path::PathBuf::from(&auth_json_path);
        let dst = project_dir.join("auth.json");
        if src.exists() {
            log.push(LogEntry::cmd(format!("Copying auth.json → {}", dst.display())));
            match tokio::fs::copy(&src, &dst).await {
                Ok(_)  => log.push(LogEntry::ok("auth.json copied")),
                Err(e) => log.push(LogEntry::warn(format!("auth.json copy failed: {}", e))),
            }
        } else {
            log.push(LogEntry::warn(format!("auth.json not found at {}", src.display())));
        }
    }

    // 4. /etc/hosts entry
    log.push(LogEntry::cmd(format!("Adding {} to /etc/hosts", local_host)));
    match tokio::fs::read_to_string("/etc/hosts").await {
        Ok(hosts) if hosts.contains(&local_host) =>
            log.push(LogEntry::info(format!("{} already in /etc/hosts", local_host))),
        _ => {
            let line = format!("127.0.0.1    {}\n", local_host);
            match crate::sudo_prompt::sudo_tee_append_with_password(&password, "/etc/hosts", &line).await {
                Ok(_)  => log.push(LogEntry::ok(format!("{} added to /etc/hosts", local_host))),
                Err(e) => {
                    log.push(LogEntry::err(format!("Failed to update /etc/hosts: {}", e)));
                    ok = false;
                }
            }
        }
    }

    // 5. VirtualHost block
    log.push(LogEntry::cmd(format!("Appending VirtualHost block to {}", apache_conf)));
    let vhost = build_vhost_block(&local_host, doc_root.to_str().unwrap_or(""), &project_name);
    match crate::sudo_prompt::sudo_tee_append_with_password(&password, &apache_conf, &vhost).await {
        Ok(_)  => log.push(LogEntry::ok("VirtualHost block appended")),
        Err(e) => {
            log.push(LogEntry::err(format!("Failed to write Apache config: {}", e)));
            ok = false;
        }
    }

    // 6. a2ensite + reload
    if ok {
        let conf_name = std::path::Path::new(&apache_conf)
            .file_stem().unwrap_or_default().to_string_lossy().to_string();
        log.push(LogEntry::cmd(format!("a2ensite {}", conf_name)));
        match crate::sudo_prompt::sudo_cmd_with_password(&password, &["a2ensite", &conf_name]).await {
            Ok(_)  => log.push(LogEntry::ok(format!("a2ensite {} done", conf_name))),
            Err(e) => log.push(LogEntry::warn(format!("a2ensite: {}", e))),
        }
        log.push(LogEntry::cmd("Reloading Apache"));
        match crate::sudo_prompt::sudo_cmd_with_password(&password, &["systemctl", "reload", "apache2"]).await {
            Ok(_)  => log.push(LogEntry::ok("Apache reloaded")),
            Err(e) => { log.push(LogEntry::err(format!("Reload failed: {}", e))); ok = false; }
        }
    }

    (log, ok)
}

fn build_vhost_block(server_name: &str, doc_root: &str, slug: &str) -> String {
    format!(
        "\n# Added by DevPanel — ApacheTouch\n\
         <VirtualHost *:80>\n\
             ServerName {sn}\n\
             DocumentRoot {dr}\n\n\
             <Directory {dr}>\n\
                 Options Indexes FollowSymLinks\n\
                 AllowOverride All\n\
                 Require all granted\n\
             </Directory>\n\n\
             ErrorLog ${{APACHE_LOG_DIR}}/{slug}_error.log\n\
             CustomLog ${{APACHE_LOG_DIR}}/{slug}_access.log combined\n\
         </VirtualHost>\n",
        sn = server_name, dr = doc_root, slug = slug,
    )
}
