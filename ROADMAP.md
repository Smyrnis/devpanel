# DevPanel — Roadmap

### Version (v0.9)

### Medium-term UX

**Keyboard navigation**
- `Ctrl+R` triggers the Refresh action on the current tab
- `Enter` in the Add VHost form submits it
- `Escape` closes any open form or modal

## UI 

### Visual polish

**Typography hierarchy**
- Section headers: 14 px, `TEXT_SECONDARY`, uppercase tracking
- Card titles: 16 px, `TEXT_PRIMARY`, weight 500
- Body / descriptions: 13 px, `TEXT_SECONDARY`
- Code paths: 12 px, monospace, `TEXT_MUTED` on `BG_SURFACE` background

**Responsive sidebar**
- Below 900 px width: sidebar collapses to icon-only mode (48 px wide)
- Hovering an icon shows a tooltip with the tab name
- Add proper icons with the help of new rust crate [iced_lucide](https://crates.io/crates/iced_lucide).

---

# After V2

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

