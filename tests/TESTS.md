# DevPanel — Test Suite Documentation

## Overview

The test suite is entirely made up of **unit and integration tests** that run
without any system services, network access, or filesystem side-effects
(except the `tests/config/config_test.rs` group which uses `tempfile` for a throwaway
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

# Run a single grouped test target
cargo test --test vhosts

# Run a single test by name
cargo test round_trip_parse_build_parse_is_stable

# Run tests matching a pattern
cargo test parse_vhosts

# Run in dry-run mode (default — no production flag needed)
cargo test
```

---

## Layout

Integration tests are grouped by area, with small root wrapper files so Cargo
still discovers each group as a normal integration test target:

```text
tests/
  config.rs      -> tests/config/config_test.rs
  core.rs        -> tests/core/dry_run_test.rs
  db.rs          -> tests/db/db_test.rs
  setup.rs       -> tests/setup/setup_log_test.rs
  system.rs      -> tests/system/system_test.rs
  vhosts.rs      -> tests/vhosts/vhosts_backend_test.rs
```

---

## Test files

### `tests/vhosts/vhosts_backend_test.rs` — 16 tests

Tests the two pure functions that do all the real work in the VHosts tab:
`parse_vhosts_from_content` and `build_conf_content`.

| Test name | What it verifies |
|---|---|
| `parse_empty_string_returns_empty` | Empty input → empty result |
| `parse_comment_only_returns_empty` | Comment-only file → empty result |
| `parse_single_vhost_no_php` | Basic ServerName + DocumentRoot round-trip |
| `parse_single_vhost_with_php_pinned` | `SetHandler x-httpd-php8.2` is extracted |
| `parse_https_entry_sets_flag_once` | HTTPS VHost blocks mark the entry as HTTPS |
| `parse_multiple_vhosts_assigns_sequential_indexes` | Three VHosts → indexes 0, 1, 2 |
| `parse_vhost_missing_servername_is_skipped` | VHost without `ServerName` is silently dropped |
| `parse_vhost_case_insensitive_directives` | `<virtualhost>` / `servername` lowercase is accepted |
| `build_empty_entries_produces_header_only` | Empty list → header comment only |
| `build_single_entry_no_php` | No `SetHandler` when php_version is None |
| `build_single_entry_with_php_produces_sethandler` | `SetHandler application/x-httpd-php8.2` present |
| `build_https_entry_produces_port_443_block` | HTTPS entries render a port 443 block |
| `build_uses_dot_to_underscore_slug_for_log_paths` | `my.project.local` → `my_project_local_error.log` |
| `round_trip_parse_build_parse_is_stable` | build → parse → same data; server_name, docroot, php all match |
| `round_trip_trailing_slash_stripped_from_server_name` | `slash.local/` → stored as `slash.local` |
| `vhost_entry_is_partial_eq` | `VHostEntry` equality compares expected fields |

**Critical paths covered:**
- The parse/build cycle must be a stable identity transformation.
- PHP version injection via `SetHandler` must survive round-trips.
- Malformed entries (missing ServerName) must not corrupt the list.

---

### `tests/setup/setup_log_test.rs` — 17 tests

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

### `tests/config/config_test.rs` — 4 tests

Tests `DevPanelConfig::load` and `save` using a temporary `HOME` directory
so no real files are touched.

| Test name | What it verifies |
|---|---|
| `load_all_keys_present` | Config keys read from TOML |
| `load_missing_file_uses_defaults` | Missing config → default values |
| `load_hosts_file_defaults_to_etc_hosts` | Absent `hosts_file` key → `/etc/hosts` |
| `save_and_reload_is_identity` | `save()` then `load()` returns same values |

---

### `tests/core/dry_run_test.rs` — 5 tests

| Test name | What it verifies |
|---|---|
| `active_is_true_without_production_feature` | Default build → dry-run active |
| `is_production_is_false_without_production_feature` | Default build → not production |
| `mode_label_dev_without_production_feature` | Label string is `"DEV (dry-run)"` |
| `log_does_not_panic` | `dry_run::log()` safe with any content |
| `log_with_long_message_does_not_panic` | 10 000-char message does not panic |

---

### `tests/system/system_test.rs` — 8 tests

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

### `tests/db/db_test.rs` — 26 tests  *(requires SQLite feature)*

Tests the in-memory SQLite settings store (`core/db.rs`).

| Group | Tests | What they verify |
|---|---|---|
| get/set | 7 | CRUD, overwrite, delete, defaults |
| Boolean helpers | 5 | `set_bool` / `get_bool` with missing keys |
| Numeric helpers | 3 | `set_u32` / `get_u32` / corrupt value fallback |
| `all_settings` | 2 | Empty DB, multiple pairs ordered by key |
| VHost metadata | 5 | Insert, overwrite, missing values, notification history |
| `UserSettings` | 4 | Defaults on fresh DB, default parity, save→load identity, partial DB |

---

## Test count summary

| Target | File | Tests |
|---|---|---|
| `vhosts` | `tests/vhosts/vhosts_backend_test.rs` | 16 |
| `setup` | `tests/setup/setup_log_test.rs` | 17 |
| `config` | `tests/config/config_test.rs` | 4 |
| `core` | `tests/core/dry_run_test.rs` | 5 |
| `system` | `tests/system/system_test.rs` | 8 |
| `db` | `tests/db/db_test.rs` | 26 |
| **Total** |  | **76** |

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
