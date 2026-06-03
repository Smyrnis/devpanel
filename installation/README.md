# Installation Scripts

`installation/` contains the structured first-run/setup installer used by DevPanel.
The in-app first-run window and Rust installer workflow live under
`installation/first_run/` and are imported by the app through `crate::installer`
so installer behavior stays separate from the normal app UI and domain modules.

## Entry Point

- `devpanel-setup.sh` is the main structured setup entrypoint used by package
  installation.
- `devpanel-create-projects-dir.sh` is the minimal in-app first-run setup
  entrypoint. It only creates the user's projects directory.

## Dependency Scripts

Scripts under `installation/dependencies/` own one setup area each:

- `common.sh` - shared package helpers.
- `install_apache.sh` - Apache packages and service setup.
- `install_php.sh` - PHP-FPM and Apache proxy setup.
- `install_mysql.sh` - MySQL packages and service setup.
- `setup_vhost.sh` - default DevPanel virtual host setup.
- `install_tools.sh` - shared project directory and support-tool setup helpers.

## Library Scripts

Scripts under `installation/lib/` are shared infrastructure:

- `context.sh` - install context and option parsing.
- `log.sh` - setup logging helpers.
- `paths.sh` - Debian/Ubuntu path constants used by shell setup.
- `runner.sh` - command execution helpers.

## Relationship To `scripts/`

`scripts/` is reserved for Debian maintainer scripts and compatibility entrypoints.

Keep installer logic in `installation/`. Keep package lifecycle hooks in `scripts/`.

## Path Constants

Shell paths live in `installation/lib/paths.sh`.

Rust paths live in `src/core/paths/debian.rs`.

When changing one, check the other in the same commit to avoid drift between app runtime behavior and install-time behavior.
