use devpanel::tabs::repos::backend::{
    extract_bitbucket_user_from_ssh_config, extract_json_str, extract_ssh_username,
    parse_gh_json, split_json_objects,
};
use devpanel::tabs::repos::Provider;

#[test] fn split_empty_string_returns_nothing() { assert!(split_json_objects("").is_empty()); }
#[test] fn split_single_object() { let objs = split_json_objects(r#"{"name":"foo"}"#); assert_eq!(objs.len(), 1); }
#[test] fn split_two_adjacent_objects() { let objs = split_json_objects(r#"{"a":1},{"b":2}"#); assert_eq!(objs.len(), 2); }
#[test] fn split_nested_braces_counted_correctly() { let objs = split_json_objects(r#"{"outer":{"inner":"val"}}"#); assert_eq!(objs.len(), 1); assert!(objs[0].contains("inner")); }
#[test] fn extract_simple_string_field() { let obj = r#"{"name":"myrepo","sshUrl":"git@github.com:user/myrepo.git"}"#; assert_eq!(extract_json_str(obj, "name").as_deref(), Some("myrepo")); }
#[test] fn extract_missing_key_returns_none() { assert!(extract_json_str(r#"{"name":"x"}"#, "missing").is_none()); }
#[test] fn extract_escaped_quote_in_value() { let obj = r#"{"desc":"say \"hello\""}"#; assert_eq!(extract_json_str(obj, "desc").as_deref(), Some(r#"say "hello""#)); }
#[test] fn extract_ignores_non_string_values() { assert!(extract_json_str(r#"{"count":42,"name":"repo"}"#, "count").is_none()); }
#[test] fn parse_empty_array() { assert!(parse_gh_json("[]").is_empty()); }
#[test] fn parse_single_repo() {
    let json = r#"[{"name":"devpanel","sshUrl":"git@github.com:user/devpanel.git","nameWithOwner":"user/devpanel"}]"#;
    let repos = parse_gh_json(json);
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "devpanel");
    assert!(matches!(repos[0].provider, Provider::GitHub));
    assert!(!repos[0].is_cloned);
}
#[test] fn parse_repo_missing_ssh_url_is_skipped() { assert!(parse_gh_json(r#"[{"name":"incomplete","nameWithOwner":"u/incomplete"}]"#).is_empty()); }
#[test] fn parse_repo_missing_name_is_skipped() { assert!(parse_gh_json(r#"[{"sshUrl":"git@github.com:u/r.git","nameWithOwner":"u/r"}]"#).is_empty()); }
#[test] fn extract_github_hi_format() { assert_eq!(extract_ssh_username("hi octocat! you've successfully authenticated"), "@octocat"); }
#[test] fn extract_bitbucket_logged_in_format() { assert_eq!(extract_ssh_username("logged in as atlassian."), "@atlassian"); }
#[test] fn extract_unknown_format_returns_connected() { assert_eq!(extract_ssh_username("some other message"), "connected"); }
#[test] fn extract_empty_string_returns_connected() { assert_eq!(extract_ssh_username(""), "connected"); }
#[test] fn extract_bb_user_from_config() {
    let config = "Host bitbucket.org\n    User git\n    IdentityFile ~/.ssh/id_ed25519\n";
    assert_eq!(extract_bitbucket_user_from_ssh_config(config).as_deref(), Some("git"));
}
#[test] fn extract_bb_user_not_present_returns_none() {
    let config = "Host github.com\n    User git\n";
    assert!(extract_bitbucket_user_from_ssh_config(config).is_none());
}
#[test] fn extract_bb_user_stops_at_next_host_block() {
    let config = "Host bitbucket.org\n    User bbuser\nHost github.com\n    User ghuser\n";
    assert_eq!(extract_bitbucket_user_from_ssh_config(config).as_deref(), Some("bbuser"));
}
#[test] fn remote_repo_is_partial_eq() {
    use devpanel::tabs::repos::RemoteRepo;
    let a = RemoteRepo { name: "r".into(), full_name: "u/r".into(), ssh_url: "git@github.com:u/r.git".into(), provider: Provider::GitHub, is_cloned: false, is_cloning: false, is_dirty: false };
    let b = a.clone();
    assert_eq!(a, b);
}
