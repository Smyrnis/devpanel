// src/tabs/repos/backend.rs — SSH connectivity checks, repo listing, git clone

use super::{Provider, RemoteRepo};

pub struct SshCheckResult {
    pub github_ok:  bool,
    pub github_msg: String,
    pub bb_ok:      bool,
    pub bb_msg:     String,
}

pub async fn check_ssh() -> SshCheckResult {
    let (github_ok, github_msg) = check_ssh_host("git@github.com").await;
    let (bb_ok, bb_msg)         = check_ssh_host("git@bitbucket.org").await;
    SshCheckResult { github_ok, github_msg, bb_ok, bb_msg }
}

pub async fn fetch_remote_repos(repos_root: String) -> Vec<RemoteRepo> {
    let mut repos: Vec<RemoteRepo> = Vec::new();
    repos.extend(fetch_github_repos().await);
    repos.extend(fetch_bitbucket_repos().await);

    // Mark repos that are already cloned locally
    for repo in &mut repos {
        let local = std::path::PathBuf::from(&repos_root).join(&repo.name);
        repo.is_cloned = local.exists();
    }

    // Uncloned first, then alphabetical within each group
    repos.sort_by(|a, b| match (a.is_cloned, b.is_cloned) {
        (true, false) => std::cmp::Ordering::Greater,
        (false, true) => std::cmp::Ordering::Less,
        _             => a.name.cmp(&b.name),
    });
    repos
}

pub async fn clone_repo(
    ssh_url:    String,
    name:       String,
    repos_root: String,
) -> (bool, String, String) {
    let dest = std::path::PathBuf::from(&repos_root).join(&name);
    if dest.exists() {
        return (false, format!("{} already exists in projects", name), ssh_url);
    }
    let out = tokio::process::Command::new("git")
        .args(["clone", &ssh_url, dest.to_string_lossy().as_ref()])
        .output()
        .await;
    match out {
        Ok(o) if o.status.success() =>
            (true, format!("Cloned {} into ~/projects/{}", name, name), ssh_url),
        Ok(o) => {
            let msg = String::from_utf8_lossy(&o.stderr)
                .trim().lines().last().unwrap_or("clone failed").to_string();
            (false, format!("Clone failed: {}", msg), ssh_url)
        }
        Err(e) => (false, format!("git not found: {}", e), ssh_url),
    }
}

async fn check_ssh_host(host: &str) -> (bool, String) {
    let out = tokio::process::Command::new("ssh")
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=8",
               "-o", "StrictHostKeyChecking=no", "-T", host])
        .output().await;

    match out {
        Ok(o) => {
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&o.stderr).to_lowercase(),
                String::from_utf8_lossy(&o.stdout).to_lowercase(),
            );
            let ok = combined.contains("hi ")
                || combined.contains("logged in as")
                || combined.contains("successfully authenticated")
                || o.status.success();
            if ok {
                (true, extract_ssh_username(&combined))
            } else {
                let first = combined.trim().lines().next().unwrap_or("no key").to_string();
                (false, first)
            }
        }
        Err(e) => (false, e.to_string()),
    }
}

fn extract_ssh_username(msg: &str) -> String {
    if let Some(after) = msg.split("hi ").nth(1) {
        let name = after.split(['!', ' ', '\n']).next().unwrap_or("").trim();
        if !name.is_empty() { return format!("@{}", name); }
    }
    if let Some(after) = msg.split("logged in as ").nth(1) {
        let name = after.split(['.', ' ', '\n']).next().unwrap_or("").trim();
        if !name.is_empty() { return format!("@{}", name); }
    }
    "connected".to_string()
}

async fn fetch_github_repos() -> Vec<RemoteRepo> {
    if let Some(repos) = try_gh_cli().await { return repos; }
    if let Some(user)  = get_github_username_via_ssh().await {
        return fetch_github_via_ls_remote(&user).await;
    }
    Vec::new()
}

async fn try_gh_cli() -> Option<Vec<RemoteRepo>> {
    let out = tokio::process::Command::new("gh")
        .args(["repo", "list", "--json", "name,sshUrl,nameWithOwner", "--limit", "200"])
        .output().await.ok()?;
    if !out.status.success() { return None; }
    let json = String::from_utf8_lossy(&out.stdout);
    if json.trim().is_empty() || json.trim() == "[]" { return None; }
    let repos = parse_gh_json(&json);
    if repos.is_empty() { None } else { Some(repos) }
}

fn parse_gh_json(json: &str) -> Vec<RemoteRepo> {
    let mut repos = Vec::new();
    let inner = json.trim().trim_start_matches('[').trim_end_matches(']');
    for obj in split_json_objects(inner) {
        let name      = extract_json_str(&obj, "name").unwrap_or_default();
        let full_name = extract_json_str(&obj, "nameWithOwner").unwrap_or_default();
        let ssh_url   = extract_json_str(&obj, "sshUrl").unwrap_or_default();
        if name.is_empty() || ssh_url.is_empty() { continue; }
        repos.push(RemoteRepo {
            name,
            full_name: if full_name.is_empty() { ssh_url.clone() } else { full_name },
            ssh_url,
            provider:   Provider::GitHub,
            is_cloned:  false,
            is_cloning: false,
        });
    }
    repos
}

async fn get_github_username_via_ssh() -> Option<String> {
    let out = tokio::process::Command::new("ssh")
        .args(["-o", "BatchMode=yes", "-o", "ConnectTimeout=8", "-T", "git@github.com"])
        .output().await.ok()?;
    let stderr = String::from_utf8_lossy(&out.stderr).to_lowercase();
    let after  = stderr.split("hi ").nth(1)?;
    let name   = after.split(['!', ' ']).next()?.trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

// Without the gh CLI there is no way to list all repos via SSH alone.
// This thing is a pile of dog shit , does not work. 
async fn fetch_github_via_ls_remote(_username: &str) -> Vec<RemoteRepo> { Vec::new() }

async fn fetch_bitbucket_repos() -> Vec<RemoteRepo> {
    scan_local_for_bitbucket().await
}

async fn scan_local_for_bitbucket() -> Vec<RemoteRepo> {
    let home = home_dir();
    let ssh_cfg = home.join(".ssh").join("config");
    if let Ok(content) = tokio::fs::read_to_string(&ssh_cfg).await {
        if let Some(user) = extract_bitbucket_user_from_ssh_config(&content) {
            if let Some(repos) = try_bitbucket_api(&user).await {
                return repos;
            }
        }
    }
    Vec::new()
}

async fn try_bitbucket_api(username: &str) -> Option<Vec<RemoteRepo>> {
    let token = std::env::var("BITBUCKET_TOKEN").ok()?;
    let url   = format!(
        "https://api.bitbucket.org/2.0/repositories/{}?pagelen=100&role=member",
        username
    );
    let out = tokio::process::Command::new("curl")
        .args(["-s", "-u", &format!("{}:{}", username, token), &url])
        .output().await.ok()?;
    if !out.status.success() { return None; }
    let json = String::from_utf8_lossy(&out.stdout);
    let repos = parse_bitbucket_json(&json, username);
    if repos.is_empty() { None } else { Some(repos) }
}

fn parse_bitbucket_json(json: &str, username: &str) -> Vec<RemoteRepo> {
    let mut repos = Vec::new();
    if let Some(start) = json.find("\"values\"") {
        for obj in split_json_objects(&json[start..]) {
            let slug      = extract_json_str(&obj, "slug").unwrap_or_default();
            let full_name = extract_json_str(&obj, "full_name").unwrap_or_default();
            if slug.is_empty() { continue; }
            repos.push(RemoteRepo {
                name:      slug.clone(),
                full_name: if full_name.is_empty() { format!("{}/{}", username, slug) } else { full_name },
                ssh_url:   format!("git@bitbucket.org:{}/{}.git", username, slug),
                provider:  Provider::Bitbucket,
                is_cloned:  false,
                is_cloning: false,
            });
        }
    }
    repos
}

fn extract_bitbucket_user_from_ssh_config(content: &str) -> Option<String> {
    let mut in_bb = false;
    for line in content.lines() {
        let t = line.trim().to_lowercase();
        if t.starts_with("host") && t.contains("bitbucket") { in_bb = true; }
        if in_bb && t.starts_with("user ") { return Some(t["user ".len()..].trim().to_string()); }
        if in_bb && t.starts_with("host ") && !t.contains("bitbucket") { in_bb = false; }
    }
    None
}

fn split_json_objects(input: &str) -> Vec<String> {
    let mut objects = Vec::new();
    let mut depth   = 0i32;
    let mut start   = None;
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' => { if depth == 0 { start = Some(i); } depth += 1; }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(s) = start { objects.push(input[s..=i].to_string()); start = None; }
                }
            }
            '"' => { i += 1; while i < chars.len() { if chars[i] == '\\' { i += 1; } else if chars[i] == '"' { break; } i += 1; } }
            _ => {}
        }
        i += 1;
    }
    objects
}

fn extract_json_str(obj: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\"", key);
    let idx     = obj.find(&pattern)?;
    let after   = obj[idx + pattern.len()..].trim_start();
    let after   = after.strip_prefix(':')?.trim_start();
    if !after.starts_with('"') { return None; }
    let mut result = String::new();
    let mut chars  = after[1..].chars();
    loop {
        match chars.next()? {
            '\\' => match chars.next()? {
                '"'  => result.push('"'),
                '\\' => result.push('\\'),
                'n'  => result.push('\n'),
                c    => { result.push('\\'); result.push(c); }
            },
            '"' => break,
            c   => result.push(c),
        }
    }
    Some(result)
}

fn home_dir() -> std::path::PathBuf {
    std::env::var("HOME").map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::path::PathBuf::from("/root"))
}
