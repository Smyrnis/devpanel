# DevPanel

Desktop GUI for managing your local dev environment on **Debian / Ubuntu**.

## What it does

- **Dashboard** — Start / Stop / Restart Apache, MySQL and PHP. Switch PHP versions. Quick links to localhost, phpMyAdmin, config files.
- **VirtualHost** — Type a project name and it sets up Apache, `/etc/hosts`, `.env` and `auth.json` automatically.
- **SSH Keys** — Generate and manage SSH keys without touching the terminal.

## Install

```bash
sudo apt install libgtk-3-dev
cargo build --release
```

## Run

```bash
./target/release/devpanel
```

## Requirements

- Debian / Ubuntu
- Rust 1.75+
- Apache2, MySQL, PHP installed
- `sudo` access
