# Runtime Resources

`share/` contains files shipped with DevPanel and loaded at runtime.

## Languages

Language JSON files live in `share/languages/`.

The Rust key map lives in `src/lang/lang_map.rs`. The JSON files are runtime-editable resources; the Rust map is the typed key accessor used by code.

## Themes

Theme JSON files live in `share/themes/`.

Theme keys are mapped under `src/core/theme/theme_map.rs`. JSON values can change without recompiling when the app loads a theme file.

## Web Assets

`share/index.php` is the default local web page installed for DevPanel.

If more web assets are added later, we should prefer moving them under `share/web/`.

## Icons

Application icon assets live in `share/icon/`.
