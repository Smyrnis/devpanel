# Runtime Resources

`share/` contains files shipped with DevPanel and loaded at runtime.

## Languages

Language JSON files live in `share/languages/`.

The Rust key map lives in `src/lang/lang_map.rs`. The JSON files are runtime-editable resources; the Rust map is the typed key accessor used by code.

## Themes

Theme JSON files live in `share/themes/`.

Theme keys are mapped under `src/core/theme/theme_map.rs`. JSON values can change without recompiling when the app loads a theme file.

## UI Configuration

`share/ui/config.json` contains runtime-editable application UI configuration
such as initial window dimensions, common UI sizing metrics, and layout
breakpoints.

The Settings tab writes user overrides to `~/.config/devpanel/ui/config.json`
so packaged defaults under `/usr/share/devpanel/ui/` and repository defaults
stay unchanged. `DEVPANEL_UI_CONFIG` can point the app at a specific UI config
file during development or testing.

## Version Metadata

`share/versions/php.json` contains runtime-editable PHP version metadata used
by first-run setup, Settings, and PHP & Tools.

Rust accessors live in `src/core/app_config.rs`. Keep distro-specific paths in
`src/core/paths/` and installer scripts; keep cross-distro product metadata in
`share/versions/`.

## Web Assets

`share/index.php` is the default local web page installed for DevPanel.

If more web assets are added later, we should prefer moving them under `share/web/`.

## Icons

Application icon assets live in `share/icon/`.
