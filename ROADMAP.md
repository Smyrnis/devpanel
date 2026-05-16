# DevPanel — Roadmap

## Medium-term features (v0.7)

### Notification system

Replace the current single-slot toast with a proper notification queue.

- Stack multiple toasts (max 3 visible at once)
- Each toast has its own 4-second countdown (configurable)
- "Dismiss all" button
- Persist the last N notifications in the SQLite `notifications` table so the user can review them later

Implementation: add a `Vec<Toast>` to `App`, render each as an absolutely-positioned card in a `stack![]`, use `iced::time::every` to tick them down.

---

### First-run improvements

- Detect whether Apache/MySQL are already installed before offering to install them
- Show a progress log in real time during installation (stream setup.log via async file watcher)
- Allow the user to skip individual packages, if there are not core packages that the app actually functions upon.

---

### SSH Keys tab enhancements

- **Key expiry** — show the creation date of each key (read from `ssh-keygen -l -f <key>`)
- **Copy public key** — one-click copy of `~/.ssh/<name>.pub` to clipboard
- **SSH agent status** — show which keys are currently loaded in `ssh-agent`

---

### Tools tab enhancements

- **Composer** — install/update Composer globally; show current version
- **Node.js** — detect installed Node/npm versions; offer nvm integration
- **Redis** — detect, start/stop, and show memory usage
- **Search across all modules** — add a filter input that searches php extensions, apache modules, and installed tools simultaneously

---

## Long-term features (v0.8)

## Scripts

- Rename `/script` to `/installation` as it is the installation scripts but keep the `/script` dir.
- Clean the `devpanel-script.sh` , there is no needs for echo only for error handlng.
- Split the `devpanel-script.sh` into a better stracture:
```
/installation/dependencies
      install_apache.sh
      install_php.sh
      install_mysql.sh
      setup_vhost.sh
      install_tools.sh
      common.sh
```

- In `/script` move the sudo running commands that are used accross the application, into a split architecture:
```
apache_sudo.sh
php_sudo.sh
mysql_sudo.sh
vhost_sudo.sh
tools_sudo.sh
common_sudo.sh

```

---

### Version (v0.9)

## UI / UX modernisation

### Immediate wins

| Item | Current | Proposed |
|---|---|---|
| Empty sidebar icon | Text `""` placeholder | Real icon via SVG path embedded in `theme.rs` or a bundled icon font |
| Toast position | Top banner (shifts all content down) | Fixed overlay in bottom-right corner, no content shift |
| VHost card layout | Full-width cards | 2-column grid on wide windows (≥1200 px) |
| Confirm delete | Inline "Confirm / Cancel" buttons on the card | Modal dialog that names the VHost being deleted |
| Active tab indicator | Background fill only | Left-border accent line (3 px, TEAL) matching the logo mark |
| PHP picker placeholder | Dropdown with raw versions | Add a suffix chip: `8.2 (active)` for the currently-active version |

### Medium-term UX

**Keyboard navigation**
- `Ctrl+1` … `Ctrl+5` switch tabs
- `Ctrl+R` triggers the Refresh action on the current tab
- `Enter` in the Add VHost form submits it
- `Escape` closes any open form or modal


**Search / command palette**
- `Ctrl+K` opens a fuzzy-search overlay
- Searches across: VHosts (by server name), repos (by name)

**Dark / light theme toggle**
- Currently hardcoded to `Theme::Dark`
- Add a `ui.theme` key to `UserSettings` (`"dark"` / `"light"` / `"system"`)
- On `"system"`, read `$GTK_THEME` or the `org.gnome.desktop.interface.color-scheme` dconf key

**Responsive sidebar**
- Below 900 px width: sidebar collapses to icon-only mode (48 px wide)
- Hovering an icon shows a tooltip with the tab name
- Below 750 px: sidebar hides entirely; a hamburger button at the top-left opens it as a drawer overlay
- Add proper icons with the help of new rust crate.

### Version (v0.9.1)

### Small Architectural improvements

- Remove the styling colors that are spread into the views , and make all of the view to take colors from the `themes.rs` file. Further color additions will be added there.
- Remove the hardcoded paths from all files and move them inside `/core/paths/<folders for each distro>` .

### Version (v0.9.2)

### Visual polish

**Cards**
- Add a subtle left-border accent (2 px) matching the service colour (green for running Apache, red for stopped, blue for PHP info)
- On hover, animate the border from 2 px to 4 px using `iced::time::every` ticks (≈ 60 fps)

**Typography hierarchy**
- Section headers: 14 px, `TEXT_SECONDARY`, uppercase tracking
- Card titles: 16 px, `TEXT_PRIMARY`, weight 500
- Body / descriptions: 13 px, `TEXT_SECONDARY`
- Code paths: 12 px, monospace, `TEXT_MUTED` on `BG_SURFACE` background

**Transition animations**
- Tab switches: fade the content in over 150 ms by tracking an `opacity: f32` on `App` and ticking it via subscription
- Toast slide-in from bottom-right using a Y-transform


---

## Very-Long term changes (v1.0)

### UI 

- Fully redesign the UI to match the moving of the application under the Elise Organization
- Colors will be updated to clean white colors scheme with dark mode support.
- Main Color will be Blue

## Main Architecture improvements

### Error handling

Replace `(bool, String)` return tuples throughout `backend.rs` files with a proper `Result<String, DevPanelError>` type:

```rust
// src/core/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DevPanelError {
    #[error("sudo command failed: {0}")]
    Sudo(String),
    #[error("file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("apache configuration error: {0}")]
    Apache(String),
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),
}
```

This makes the error paths type-safe and lets `?` propagation work inside async functions.

### State machine for service actions

The current `trigger_sudo → dispatch_sudo_action` chain is correct but the `PendingAction` enum grows linearly with every new feature. Refactor to a `Command` trait:

```rust
pub trait SudoCommand: Send + 'static {
    fn execute(self: Box<Self>, password: &str) -> Pin<Box<dyn Future<Output = Message> + Send>>;
}
```

Each action becomes a struct implementing `SudoCommand`. The `SudoModal` stores a `Box<dyn SudoCommand>`.

### Split `update.rs` further

`update.rs` at 972 lines is already split by tab but each handler is still a large `match` arm. Consider extracting each handler into its own file:

```
src/app/
  mod.rs
  update.rs          ← only the top-level dispatch
  handlers/
    dashboard.rs
    tools.rs
    vhosts.rs
    repos.rs
    ssh_keys.rs
    sudo.rs
    first_run.rs
    config.rs        ← new
```

### Async file watching

Use `notify` crate to watch `devpanel.conf` for external changes and auto-reload the VHosts list when the file is modified outside the app.

```toml
notify = "6"
```

Wire a `Subscription` that wraps a `notify::Watcher` into an Iced `Subscription` via a channel.



# After V2
### Goal of the Roadmap 

**General**
- Make the change of a color, font e.t.c to be made very easy and simple through the change of one line .
- Make the Architecture of the application to be simple, easy navigated straight forward, and easy expandable (The end goal of it is to remind something like an MVC).
- Make easy to the eye of the user with easy controls.

### Later enchancements
**Script Health Check**
- Verify system readiness (ports, services, permissions e.t.c)
**Rollback support**
- If install or action fails, revert partial changes.
**Environment detection**
- Detect distros and branch logic (Problem: do not know how to detect needed file paths.)
**Controlled Concurrency**
- limit number of parallel jobs

**Notification**
- Implement support for system notification through distro.

**Tray**
- Implement tray support that will have limiting functionality (example: restart services, open localhost e.t.c), that the user can add as slots.

