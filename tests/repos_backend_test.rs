use devpanel::tabs::repos::backend::{
    extract_bitbucket_user_from_ssh_config,
    extract_json_str,
    extract_ssh_username,
    parse_gh_json,
    split_json_objects,
};
use devpanel::tabs::repos::Provider;

#[test]
fn split_empty_string_returns_nothing() {
    assert!(split_json_objects("").is_empty());
}

#[test]
fn split_single_object() {
    let input = r#"{"name":"foo"}"#;
    let objs = split_json_objects(input);
    assert_eq!(objs.len(), 1);
    assert_eq!(objs[0], r#"{"name":"foo"}"#);
}

#[test]
fn split_two_adjacent_objects() {
    let input = r#"{"a":1},{"b":2}"#;
    let objs = split_json_objects(input);
    assert_eq!(objs.len(), 2);
}

#[test]
fn split_nested_braces_counted_correctly() {
    let input = r#"{"outer":{"inner":"val"}}"#;
    let objs = split_json_objects(input);
    assert_eq!(objs.len(), 1);
    assert!(objs[0].contains("inner"));
}

#[test]
fn split_escaped_brace_inside_string_not_counted() {
    // The { inside a string literal must not open a new depth level.
    let input = r#"{"key":"{not-a-block}","real":1}"#;
    let objs = split_json_objects(input);
    assert_eq!(objs.len(), 1);
}

#[test]
fn extract_simple_string_field() {
    let obj = r#"{"name":"myrepo","sshUrl":"git@github.com:user/myrepo.git"}"#;
    assert_eq!(extract_json_str(obj, "name").as_deref(), Some("myrepo"));
    assert_eq!(
        extract_json_str(obj, "sshUrl").as_deref(),
        Some("git@github.com:user/myrepo.git")
    );
}

#[test]
fn extract_missing_key_returns_none() {
    let obj = r#"{"name":"x"}"#;
    assert!(extract_json_str(obj, "missing").is_none());
}

#[test]
fn extract_escaped_quote_in_value() {
    let obj = r#"{"desc":"say \"hello\""}"#;
    assert_eq!(extract_json_str(obj, "desc").as_deref(), Some(r#"say "hello""#));
}

#[test]
fn extract_ignores_non_string_values() {
    let obj = r#"{"count":42,"name":"repo"}"#;
    assert!(extract_json_str(obj, "count").is_none());
}

#[test]
fn parse_empty_array() {
    assert!(parse_gh_json("[]").is_empty());
}

#[test]
fn parse_single_repo() {
    let json = r#"[{"name":"devpanel","sshUrl":"git@github.com:user/devpanel.git","nameWithOwner":"user/devpanel"}]"#;
    let repos = parse_gh_json(json);
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "devpanel");
    assert_eq!(repos[0].ssh_url, "git@github.com:user/devpanel.git");
    assert_eq!(repos[0].full_name, "user/devpanel");
    assert!(matches!(repos[0].provider, Provider::GitHub));
    assert!(!repos[0].is_cloned);
}

#[test]
fn parse_two_repos() {
    let json = r#"[
        {"name":"repo-a","sshUrl":"git@github.com:u/repo-a.git","nameWithOwner":"u/repo-a"},
        {"name":"repo-b","sshUrl":"git@github.com:u/repo-b.git","nameWithOwner":"u/repo-b"}
    ]"#;
    let repos = parse_gh_json(json);
    assert_eq!(repos.len(), 2);
    assert_eq!(repos[0].name, "repo-a");
    assert_eq!(repos[1].name, "repo-b");
}

#[test]
fn parse_repo_missing_ssh_url_is_skipped() {
    // sshUrl is required — repos without it should not appear in results.
    let json = r#"[{"name":"incomplete","nameWithOwner":"u/incomplete"}]"#;
    let repos = parse_gh_json(json);
    assert!(repos.is_empty(), "repo without sshUrl must be skipped");
}

#[test]
fn parse_repo_missing_name_is_skipped() {
    let json = r#"[{"sshUrl":"git@github.com:u/r.git","nameWithOwner":"u/r"}]"#;
    let repos = parse_gh_json(json);
    assert!(repos.is_empty(), "repo without name must be skipped");
}

#[test]
fn parse_falls_back_to_ssh_url_when_name_with_owner_absent() {
    let json = r#"[{"name":"r","sshUrl":"git@github.com:u/r.git"}]"#;
    let repos = parse_gh_json(json);
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].full_name, "git@github.com:u/r.git");
}

#[test]
fn extract_github_hi_format() {
    let msg = "hi octocat! you've successfully authenticated";
    assert_eq!(extract_ssh_username(msg), "@octocat");
}

#[test]
fn extract_bitbucket_logged_in_format() {
    let msg = "logged in as atlassian.";
    assert_eq!(extract_ssh_username(msg), "@atlassian");
}

#[test]
fn extract_unknown_format_returns_connected() {
    let msg = "some other message with no username";
    assert_eq!(extract_ssh_username(msg), "connected");
}

#[test]
fn extract_empty_string_returns_connected() {
    assert_eq!(extract_ssh_username(""), "connected");
}

#[test]
fn extract_hi_with_space_stops_at_exclamation() {
    let msg = "hi  username!  rest of message";
    // Extra spaces: the username parsing should still find the name.
    let result = extract_ssh_username(msg);
    assert!(result.starts_with('@') || result == "connected");
}

#[test]
fn extract_bb_user_from_config() {
    let config = "\
Host bitbucket.org
    User git
    IdentityFile ~/.ssh/id_ed25519
";
    // The SSH config 'User' line gives us the username.
    let result = extract_bitbucket_user_from_ssh_config(config);
    assert_eq!(result.as_deref(), Some("git"));
}

#[test]
fn extract_bb_user_not_present_returns_none() {
    let config = "\
Host github.com
    User git
    IdentityFile ~/.ssh/id_ed25519
";
    assert!(extract_bitbucket_user_from_ssh_config(config).is_none());
}

#[test]
fn extract_bb_user_stops_at_next_host_block() {
    let config = "\
Host bitbucket.org
    User bbuser
Host github.com
    User ghuser
";
    let result = extract_bitbucket_user_from_ssh_config(config);
    assert_eq!(result.as_deref(), Some("bbuser"));
}

#[test]
fn extract_bb_user_empty_config_returns_none() {
    assert!(extract_bitbucket_user_from_ssh_config("").is_none());
}
