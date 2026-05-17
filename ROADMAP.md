# DevPanel — Roadmap

### Arhitectural changes

1. The sudo_s module is awkwardly named and structured. Having src/sudo_s/ with files like apache_sudo.rs, php_sudo.rs etc. is redundant — the directory already implies sudo. Rename it to src/operations/ or src/commands/ with apache.rs, php.rs, vhosts.rs. The common_sudo.rs file should just be mod.rs for that module since everything in it is shared infrastructure.
2. The SudoCommand trait + commands.rs file is doing too much. You have 15+ command structs in one 300-line file (src/core/sudo_prompt/commands.rs). Each command struct is essentially a closure workaround. Either split them into domain-specific files (vhost commands, tool commands, dashboard commands) or — since you're on Rust 2024 edition — consider whether async closures could replace some of the boilerplate.
3. src/app/handlers/ is a good pattern, but the handlers are tightly coupled to App. Every handler takes &mut self on the full App struct. When App grows, this becomes hard to test. Consider extracting state into domain sub-structs (you're already partway there with DashboardTab, ToolsTab etc.) and having handlers operate on those directly, with App just routing.
4. The ui/templates/view.rs is just a re-export file — this is noise. Flatten it: consumers can import from ui::templates::buttons, ui::templates::cards directly, or create a proper prelude. The current indirection adds confusion without benefit.
5. There's no clear boundary between UI state and domain state. VHostsTab, ReposTab etc. mix display state (form fields, loading booleans, status messages) with actual data. Consider splitting each tab into a Tab (UI state) and the underlying data/service layer. This also makes the 97 tests you have more useful — right now they can only test pure functions, not the state machine.
6. src/helpers/ is a grab-bag. json.rs belongs in src/core/ or alongside the repos backend that uses it (you're already re-exporting it from there). time.rs and process.rs are genuinely utility-level. env.rs duplicates what src/core/system/desktop.rs already does (get_home exists in both places).
7. Path constants scattered across two files. You have src/core/paths/debian.rs with the constants and installation/lib/paths.sh with the same paths in bash. This is unavoidable for the shell scripts, but you could generate the Rust constants from a single source, or at minimum add a doc comment on the Rust side that cross-references the shell file to avoid drift.
8. The dry_run module is fine, but its usage pattern — checking dry_run::active() inside every async function — means the production code paths are never exercised in tests. A cleaner pattern is a trait like trait CommandRunner with a DryRunRunner and RealRunner impl, injected at startup. This would let you test the full command logic without actually running sudo.
The most impactful changes in order of effort vs benefit: fix the env.rs duplication now (15 minutes), split commands.rs by domain (1 hour), rename sudo_s (30 minutes with a find/replace).

# UI redisign

Add icons from the iced_font_awesome crate[icons](https://crates.io/crates/iced_font_awesome)

Make the changes accordingly with the `images/redesign/*` there you can find images that has the redesign mockups.(The images are designed for macos create the ui for linux use.)


### Later enchancements


**Script Health Check**
- Verify system readiness (ports, services, permissions e.t.c)
**Rollback support**
- If install or action fails, revert partial changes.
**Environment detection**
- Detect distros and branch logic (Problem: do not know how to detect needed file paths.)

**Notification**
- Implement support for system notification through distro.

**Tray**
- Implement tray support that will have limiting functionality (example: restart services, open localhost e.t.c), that the user can add as slots.

