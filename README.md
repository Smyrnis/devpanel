# DevPanel 
<img width="100px" height="100px" src="icon.png" alt="application icon"/> 

A lightweight desktop GUI for managing your local PHP development environment on Ubuntu and Debian. Built with Rust and [Iced](https://github.com/iced-rs/iced).

<p> Current version: <strong> 0.5.9 </strong> </p>

---

## What it does

The view that devpanel has.

| Tab | What you get |
|---|---|
| **Dashboard** | Start, stop and restart Apache and MySQL. Switch PHP versions. Jump to common files and folders with one click. |
| **VirtualHosts** | Add, edit and delete Apache virtual hosts stored in a single `devpanel.conf` file. Edit the raw config directly in the built-in editor. |
| **SSH Keys** | Generate Ed25519, RSA and ECDSA keys. View all keys in `~/.ssh`. |
| **Repos** | Fetch and clone GitHub and Bitbucket repos over SSH. Requires `gh` CLI or a configured SSH key. |
| **Tools** | Install and remove PHP versions, toggle Apache modules, manage PHP extensions, and launch a MySQL terminal. |

---

## Requirements

- Ubuntu 22.04 / 24.04 or Debian 11+
- Apache 2.4
- MySQL 8 or MariaDB 10.6+
- PHP 8.x
- `sudo` access

---

## Installation

The installation come only as a .deb package for now. 

Download it [here](https://github.com/Smyrnis/devpanel/packages)

---

## Configuration

Settings are stored in `~/.config/devpanel/config.toml` and created automatically on first run.

```toml
repos_root    = "/home/user/projects"
devpanel_conf = "/etc/apache2/sites-available/devpanel.conf"
hosts_file    = "/etc/hosts"
```

---

# TL;DR

The application is current on development.

If you encounter problems please open an issue.

---
## For the Tests check here

[Tests](tests/TESTS.md)

## License

MIT — see [LICENSE](LICENSE).