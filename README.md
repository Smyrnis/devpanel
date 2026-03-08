# DevPanel

A lightweight desktop GUI for local PHP development on Ubuntu / Debian.\
Built with Rust and Iced.\
Pairs with `share/index.php` --- a matching dark-themed Apache welcome
page served at `http://localhost`.

------------------------------------------------------------------------

## What Works

-   Dashboard (Apache + MySQL checks, PHP switcher, shortcuts)
-   VirtualHosts (list, add, delete)
-   SSH key generation & listing
-   GitHub (`gh`) and Bitbucket repo fetch
-   Repo clone & open
-   PHP version install/remove
-   Apache module toggle
-   PHP extension management
-   DB terminal launcher
-   Session-based sudo handling
-   Config file support
-   Dark-themed Apache welcome page

------------------------------------------------------------------------

## Known Limitations

-   No inline VirtualHost editing (delete + re-add required)
-   Bitbucket limited to first 100 repos
-   No in-app config editor
-   No live auto-refresh dashboard
-   Single vhost file management
-   No database GUI (CLI only)
-   Wayland runs under XWayland

------------------------------------------------------------------------

## Requirements

-   Ubuntu 22.04 / 24.04 or Debian 11+
-   Apache 2.4
-   MySQL 8 or MariaDB 10.6+
-   PHP 8.x (`php-cli`)
-   `sudo` access
-   Terminal emulator

Optional: - `gh` CLI - `ondrej/php` PPA

------------------------------------------------------------------------

## Installation

### Build from source

``` bash
curl https://sh.rustup.rs -sSf | sh
source "$HOME/.cargo/env"

sudo apt-get install -y pkg-config libfontconfig1-dev libfreetype6-dev libx11-dev libxkbcommon-dev libvulkan-dev cmake build-essential git

git clone https://github.com/yourname/devpanel.git
cd devpanel
cargo build --release

sudo bash scripts/devpanel-setup.sh
./target/release/devpanel
```

### Install from .deb

``` bash
sudo dpkg -i devpanel_*.deb
devpanel
```

------------------------------------------------------------------------

## First Run

`devpanel-setup.sh`:

-   Creates `~/projects`
-   Configures Apache site
-   Enables `mod_rewrite`
-   Writes config file

Then open:

http://localhost

------------------------------------------------------------------------

## Configuration

`~/.config/devpanel/config.toml`

``` toml
repos_root    = "/home/user/projects"
devpanel_conf = "/etc/apache2/sites-available/devpanel.conf"
hosts_file    = "/etc/hosts"
```

No in-app settings UI.

------------------------------------------------------------------------

## License

MIT
