# DevPanel 
A lightweight desktop GUI for managing your local PHP development environment on Ubuntu and Debian. Built with Rust and [Iced](https://github.com/iced-rs/iced).

<p> Current version: <strong> 0.26.35 </strong> </p>

---

# TL;DR

The application is currently under development.
A lot of the new code that has been added has a lot of bugs and is not yet stable.

---

## It is not recommended to build from source.

---

Some-what stable version can be found packaged as .deb.


---

## What it does

The view that devpanel has.

| Tab | What you get |
|---|---|
| **Dashboard** | Manage Apache, MySQL, PHP, Composer, and Node/NVM. Jump to common files and folders with one click. |
| **VirtualHosts** | Add, edit and delete Apache virtual hosts stored in a single `devpanel.conf` file. Edit the raw config directly in the built-in editor. |
| **SSH Keys** | Generate Ed25519, RSA and ECDSA keys. View all keys in `~/.ssh`. |
| **Tools** | Manage PHP versions and extensions, Apache modules, Redis, and database terminals. |

---

## Requirements

- Ubuntu 22.04 LTS / 24.04 LTS / 26.04 LTS or Debian 11+
- Apache 2.4
- MySQL 8 or MariaDB 10.6+
- PHP 8.x
- `sudo` access

---

## Installation

Installation is currently provided as a `.deb` package.

Download it [here](https://github.com/Smyrnis/devpanel/packages)

The first-run installer offers Composer and Node through NVM as optional
components. Composer is installed globally. NVM and Node are installed for the
desktop user, not root.

Apache and PHP-FPM receive inherited read/traverse ACL access to `~/projects`.
Writable application directories such as framework caches, uploads, or storage
remain project-specific and are not made globally writable.

After installation, use the Composer and Node sections on the **Dashboard** to:

- Install or update Composer and select a Composer release channel.
- Install NVM and install, select, or set the default Node version.
- View the detected Composer, Node, npm, and NVM status.

---

## Configuration

Settings are stored in `~/.config/devpanel/config.toml` and created automatically on first run.

```toml
devpanel_conf = "/etc/apache2/sites-available/devpanel.conf"
hosts_file    = "/etc/hosts"
```

Runtime version choices are read from:

- `/usr/share/devpanel/versions/composer.json`
- `/usr/share/devpanel/versions/node.json`

---

## For the Tests check here

[Tests](tests/TESTS.md)

## License

MIT — see [LICENSE](LICENSE).
