# DevPanel — Test Suite Documentation

## Overview

The test suite is entirely made up of **unit and integration tests** that run
without any system services, network access, or filesystem side-effects
(except the `config_test.rs` group which uses `tempfile` for a throwaway
HOME directory).

No GUI is tested — Iced widget trees are not renderable in a headless test
runner and the rendering logic is thin wiring over pure data functions that
are already covered indirectly.

---

## Quick start

```bash
# Run all tests
cargo test

# Run with output visible
cargo test -- --nocapture

# Run a single test file
cargo test --test vhosts_backend_test

# Run a single test by name
cargo test round_trip_parse_build_parse_is_stable

# Run tests matching a pattern
cargo test parse_vhosts

# Run in dry-run mode (default — no production flag needed)
cargo test
```

---

## One-time setup

### 1. Add the `[lib]` section to `Cargo.toml`

Integration tests in `tests/` need to import internal modules.  Add this
block directly after `[[bin]]`:

```toml
[lib]
name = "devpanel"
path = "src/lib.rs"
```

### 2. Create `src/lib.rs`

```rust
// src/lib.rs — exposes internals to integration tests
#![allow(dead_code)]
pub mod core;
pub mod messages;
pub mod tabs;
```

### 3. Make private helpers `pub(crate)` in `src/tabs/repos/backend.rs`

Six private functions need to be visible to the test crate.  Change `fn` to
`pub(crate) fn` for:

```
extract_ssh_username
parse_gh_json
parse_bitbucket_json
extract_bitbucket_user_from_ssh_config
split_json_objects
extract_json_str
```

### 4. Add dev dependencies to `Cargo.toml`

```toml
[dev-dependencies]
tempfile = "3"
```

### 5. For the SQLite tests — add runtime dependency

```toml
[dependencies]
rusqlite           = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1.2"
```

Then add `src/core/db.rs` and re-export it from `src/core/mod.rs`:

```rust
// in src/core/mod.rs
pub mod db;
```

---

## Test files

### `tests/vhosts_backend_test.rs` — 14 tests

Tests the two pure functions that do all the real work in the VHosts tab:
`parse_vhosts_from_content` and `build_conf_content`.

| Test name | What it verifies |
|---|---|
| `parse_empty_string_returns_empty` | Empty input → empty result |
| `parse_comment_only_returns_empty` | Comment-only file → empty result |
| `parse_single_vhost_no_php` | Basic ServerName + DocumentRoot round-trip |
| `parse_single_vhost_with_php_pinned` | `SetHandler x-httpd-php8.2` is extracted |
| `parse_multiple_vhosts_assigns_sequential_indexes` | Three VHosts → indexes 0, 1, 2 |
| `parse_vhost_missing_servername_is_skipped` | VHost without `ServerName` is silently dropped |
| `parse_vhost_case_insensitive_directives` | `<virtualhost>` / `servername` lowercase is accepted |
| `parse_mixed_php_versions_in_multiple_vhosts` | PHP 7.4, 8.3, and no PHP in one file |
| `build_empty_entries_produces_header_only` | Empty list → header comment only |
| `build_single_entry_no_php` | No `SetHandler` when php_version is None |
| `build_single_entry_with_php_produces_sethandler` | `SetHandler application/x-httpd-php8.2` present |
| `build_uses_dot_to_underscore_slug_for_log_paths` | `my.project.local` → `my_project_local_error.log` |
| `build_multiple_entries_all_present` | Two VHosts both rendered; PHP only for the one that has it |
| `round_trip_parse_build_parse_is_stable` | build → parse → same data; server_name, docroot, php all match |
| `round_trip_trailing_slash_stripped_from_server_name` | `slash.local/` → stored as `slash.local` |

**Critical paths covered:**
- The parse/build cycle must be a stable identity transformation.
- PHP version injection via `SetHandler` must survive round-trips.
- Malformed entries (missing ServerName) must not corrupt the list.

---

### `tests/repos_backend_test.rs` — 18 tests

Tests the JSON parsing, SSH username extraction, and Bitbucket SSH config
helpers in `tabs/repos/backend.rs`.

| Test name | What it verifies |
|---|---|
| `split_empty_string_returns_nothing` | Empty input → no objects |
| `split_single_object` | One `{...}` → one result |
| `split_two_adjacent_objects` | Two adjacent `{...},{...}` → two results |
| `split_nested_braces_counted_correctly` | `{"outer":{"inner":"val"}}` is one object |
| `split_escaped_brace_inside_string_not_counted` | `{` inside a JSON string doesn't open depth |
| `extract_simple_string_field` | `"name"` and `"sshUrl"` extracted correctly |
| `extract_missing_key_returns_none` | Unknown key → `None` |
| `extract_escaped_quote_in_value` | `\"` inside a string value handled |
| `extract_ignores_non_string_values` | Numeric value for key → `None` |
| `parse_empty_array` | `[]` → empty vec |
| `parse_single_repo` | Name, ssh_url, full_name, provider=GitHub |
| `parse_two_repos` | Both repos present in correct order |
| `parse_repo_missing_ssh_url_is_skipped` | Required field absent → repo dropped |
| `parse_repo_missing_name_is_skipped` | Required field absent → repo dropped |
| `parse_falls_back_to_ssh_url_when_name_with_owner_absent` | `full_name` falls back to `ssh_url` |
| `extract_github_hi_format` | `"hi octocat!"` → `"@octocat"` |
| `extract_bitbucket_logged_in_format` | `"logged in as atlassian."` → `"@atlassian"` |
| `extract_unknown_format_returns_connected` | Unrecognised message → `"connected"` |
| `extract_empty_string_returns_connected` | Empty string → `"connected"` |
| `extract_bb_user_from_config` | Finds `User git` under `Host bitbucket.org` |
| `extract_bb_user_not_present_returns_none` | No bitbucket block → `None` |
| `extract_bb_user_stops_at_next_host_block` | Does not bleed into next `Host` block |
| `extract_bb_user_empty_config_returns_none` | Empty file → `None` |

---

### `tests/setup_log_test.rs` — 16 tests

Tests `SetupLogEntry::parse` against every log level and various edge cases.

| Test name | What it verifies |
|---|---|
| `parse_ok_level` | `[OK]` tag maps to `LogLevel::Ok` |
| `parse_step_level` | `[STEP]` |
| `parse_info_level` | `[INFO]` |
| `parse_warn_level` | `[WARN]` |
| `parse_error_level` | `[ERROR]` |
| `parse_cmd_level` | `[CMD]` |
| `parse_out_level` | `[OUT]` |
| `parse_postinst_level` | `[POSTINST]` |
| `parse_unknown_level_tag` | Unrecognised tag → `LogLevel::Unknown` |
| `parse_no_bracket_returns_unknown` | No bracket → `LogLevel::Unknown` |
| `parse_empty_string_returns_none` | Empty → `None` |
| `parse_whitespace_only_returns_none` | Spaces only → `None` |
| `parse_line_too_short_returns_none` | < 20 chars → `None` |
| `parse_preserves_timestamp_exactly` | 19-char timestamp extracted verbatim |
| `parse_message_with_colon` | Colon in message not misinterpreted as key-value separator |
| `parse_message_trimmed_of_whitespace` | Leading and trailing spaces stripped |
| `only_error_and_warn_are_issues` | Filter simulation matches `read_setup_issues()` logic |

---

### `tests/config_test.rs` — 9 tests

Tests `DevPanelConfig::load` and `save` using a temporary `HOME` directory
so no real files are touched.

| Test name | What it verifies |
|---|---|
| `load_all_keys_present` | All three keys read from TOML |
| `load_no_spaces_around_equals` | `key="value"` (no spaces) works |
| `load_extra_spaces_around_equals` | `key   =   "value"` works |
| `load_missing_file_uses_defaults` | Missing config → default values |
| `load_hosts_file_defaults_to_etc_hosts` | Absent `hosts_file` key → `/etc/hosts` |
| `load_ignores_lines_with_wrong_key` | `repos_root_extra` does not match `repos_root` |
| `save_and_reload_is_identity` | `save()` then `load()` returns same values |
| `save_produces_valid_toml_file` | Output file contains all keys |

---

### `tests/dry_run_test.rs` — 5 tests

| Test name | What it verifies |
|---|---|
| `active_is_true_without_production_feature` | Default build → dry-run active |
| `is_production_is_false_without_production_feature` | Default build → not production |
| `mode_label_dev_without_production_feature` | Label string is `"DEV (dry-run)"` |
| `log_does_not_panic` | `dry_run::log()` safe with any content |
| `log_with_long_message_does_not_panic` | 10 000-char message does not panic |

---

### `tests/system_test.rs` — 7 tests

| Test name | What it verifies |
|---|---|
| `shell_quote_simple_path` | `/var/www/html` → `'/var/www/html'` |
| `shell_quote_path_with_spaces` | Spaces preserved inside quotes |
| `shell_quote_path_with_single_quote` | `'` → `'\''` escape sequence |
| `shell_quote_empty_string` | Empty string → `''` |
| `shell_quote_path_with_special_chars` | `$` and `*` safe inside single quotes |
| `shell_quote_multiple_single_quotes` | Multiple quotes all escaped |
| `get_home_returns_non_empty_path` | Path is non-empty |
| `get_home_reflects_home_env_var` | Reads `$HOME` environment variable |

---

### `tests/db_test.rs` — 25 tests  *(requires SQLite feature)*

Tests the in-memory SQLite settings store (`core/db.rs`).

| Group | Tests | What they verify |
|---|---|---|
| get/set | 5 | CRUD, overwrite, delete |
| Boolean helpers | 5 | `set_bool` / `get_bool` with missing keys |
| Numeric helpers | 3 | `set_u32` / `get_u32` / corrupt value fallback |
| `all_settings` | 2 | Empty DB, multiple pairs ordered by key |
| VHost tags | 4 | Insert, overwrite, `all_vhost_meta` |
| `UserSettings` | 3 | Defaults on fresh DB, save→load identity, partial DB |

---

## Test count summary

| File | Tests |
|---|---|
| `vhosts_backend_test.rs` | 15 |
| `repos_backend_test.rs` | 18 |
| `setup_log_test.rs` | 17 |
| `config_test.rs` | 9 |
| `dry_run_test.rs` | 5 |
| `system_test.rs` | 8 |
| `db_test.rs` | 25 |
| **Total** | **97** |

---

## What is NOT tested (and why)

| Area | Reason |
|---|---|
| Iced widget rendering | No headless renderer; widget logic is thin wiring over data |
| `probe_services()` | Calls `systemctl` — mocked at the dry-run level |
| `sudo_cmd_with_password()` | Calls real `sudo` — covered by dry-run flag; manual QA only |
| `scan_php_versions()` | Probes `/usr/bin/php*` — not present in CI |
| SSH key generation | Calls `ssh-keygen` — manual QA only |
| Git clone | Calls `git clone` — network access; manual QA only |

**Principle:** test every function that can be tested without process spawning.
For functions that must shell out, the dry-run mode provides a safe no-op path
that is itself verified by `dry_run_test.rs`.

---

## CI integration

Minimal GitHub Actions workflow:

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: sudo apt-get install -y libgtk-3-dev
      - run: cargo test
```

The `libgtk-3-dev` package is required because `rfd` (the file dialog crate)
links against GTK even on headless systems.

---

## Running with coverage (optional)

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage/

# Open report
xdg-open coverage/tarpaulin-report.html
```

Expected coverage: ~75 % of lines in `core/` and `tabs/*/backend.rs`,
lower overall because `app/`, `tabs/*/view.rs`, and `core/sudo_prompt.rs`
are GUI/system code that cannot be exercised headlessly.
