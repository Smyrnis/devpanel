# First-Run Installer

This folder owns the in-app first-run setup boundary.

The normal application imports this code through `crate::installer`, but the
installer model, service, and setup window live here to keep first-run setup
separate from the main app UI and domain modules.

Current policy:

- The projects directory setup is required.
- Apache, latest PHP, MySQL, and latest PHP extras are optional.
- First-run only offers the latest PHP branch. Older PHP branches stay
  available in the main PHP & Tools workflow.
- Local development uses dry-run behavior by default.
- Production first-run uses `installation/devpanel-create-projects-dir.sh` for
  the required setup step.
