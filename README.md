# DevPanel

A lightweight desktop GUI for local PHP development on Ubuntu/Debian.  
Built with Rust and [Iced](https://github.com/iced-rs/iced).

---

## What DevPanel Does

| Feature | Description |
|---|---|
| **Dashboard** | Live Apache / MySQL status, PHP version switcher, quick-open shortcuts |
| **Repos** | Browse and clone GitHub / Bitbucket repos via SSH — one click into `~/projects/` |
| **VirtualHosts** | Add, edit, delete `.local` vhosts — writes a single `devpanel.conf` |
| **SSH Keys** | Generate ed25519 / RSA-4096 / ECDSA keys, add to ssh-agent |
| **Tools** | Install PHP versions, toggle Apache modules, install PHP extensions, open DB terminal |

---

## Requirements

| Dependency | Why |
|---|---|
| Ubuntu 22.04 / 24.04 (or Debian 11+) | Tested distros |
| Apache2 | Web server |
| MySQL 8 or MariaDB 10.6+ | Database |
| PHP 8.x (`php-cli`) | Web language |
| `sudo` access | Service control, vhost management |
| A terminal emulator | DB CLI (gnome-terminal, xterm, konsole, xfce4-terminal, etc.) |

Optional but recommended:
- `gh` CLI — needed for GitHub repo listing in the Repos tab
- `xclip` or `wl-clipboard` — clipboard support in Tools tab

---

## Installation

### Option A — Build from source

```bash
# 1. Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# 2. Install system build dependencies
sudo apt-get install -y \
    pkg-config libfontconfig1-dev libfreetype6-dev \
    libx11-dev libxkbcommon-dev libvulkan-dev \
    cmake build-essential git

# 3. Clone and build
git clone https://github.com/yourname/devpanel.git
cd devpanel
cargo build --release

# 4. Run the first-time setup
sudo bash scripts/devpanel-setup.sh

# 5. Launch
./target/release/devpanel
```

### Option B — Install from .deb package

```bash
sudo dpkg -i devpanel_*.deb
# postinst runs devpanel-setup.sh automatically
devpanel
```

---

## First-Run Setup (`devpanel-setup.sh`)

The setup script runs **once** (either automatically via `postinst` or manually) and does exactly seven things:

| Step | What Happens |
|---|---|
| 1 | Creates `~/projects/` (owned by you + `www-data` group) |
| 2 | Copies `index.php` into `~/projects/` as the Apache welcome page |
| 3 | Creates `/etc/apache2/sites-available/devpanel.conf` (empty, ready for vhosts) |
| 4 | Runs `a2ensite devpanel.conf` to enable it |
| 5 | Points Apache's `000-default.conf` `DocumentRoot` → `~/projects/` |
| 6 | Enables `mod_rewrite` |
| 7 | Writes `~/.config/devpanel/config.toml` |

After setup, open [http://localhost](http://localhost) — you'll see the DevPanel welcome page.

### What setup does NOT touch
- PHP configuration files  
- MySQL configuration files  
- Any files outside `~/projects/`, `/etc/apache2/`, and `~/.config/devpanel/`

---

## First Launch Walkthrough

### 1 — Dashboard

When you open DevPanel you land on the **Dashboard**.

```
┌─────────────────────────────────────────────────────┐
│  Apache  ● running     MySQL  ● running              │
│  PHP 8.3 (active)                                   │
│                                                     │
│  [Open localhost]  [phpMyAdmin]  [Projects Folder]  │
│                                                     │
│  Config Files                                       │
│  apache2.conf · sites-available · devpanel.conf     │
│  /etc/php · /etc/mysql · /etc/hosts · ~/projects/   │
└─────────────────────────────────────────────────────┘
```

- **Green dot** = service is running  
- **Red dot** = service is stopped — click the service name to start it  
- Click any config file shortcut to open it in your file manager or editor

---

### 2 — VirtualHosts

This is where you create per-project local domains.

**Adding a vhost:**

1. Click **VirtualHosts** in the sidebar
2. Click **+ Add VHost**
3. Fill in:
   - **Server Name** — e.g. `myapp.local`
   - **Document Root** — e.g. `/home/yourname/projects/myapp/public`
4. Click **Create**
5. Enter your sudo password when prompted
6. DevPanel:
   - Writes the `<VirtualHost>` block into `/etc/apache2/sites-available/devpanel.conf`
   - Adds `127.0.0.1  myapp.local` to `/etc/hosts`
   - Reloads Apache

Open [http://myapp.local](http://myapp.local) in your browser.

**Editing / deleting:** each vhost card has Edit and Delete buttons.  
**Open conf file:** click **Open devpanel.conf** to inspect the raw file.

---

### 3 — Repos

Connect to GitHub and Bitbucket and clone repos directly into `~/projects/`.

**Step 1 — Set up SSH keys**  
Go to **SSH Keys** tab → **Generate Key** → add the public key to:
- GitHub: [github.com/settings/keys](https://github.com/settings/keys)
- Bitbucket: Profile → Personal settings → SSH keys

**Step 2 — Check SSH**  
Back in **Repos**, click **Check SSH**.  
Pills show `● connected @username` when keys are working.

**Step 3 — Fetch repos**  
Click **Fetch Repos**. DevPanel will:
- Use the `gh` CLI for GitHub (most complete — install from [cli.github.com](https://cli.github.com/))
- Use `BITBUCKET_TOKEN` env var for Bitbucket (set `export BITBUCKET_TOKEN=your_app_password`)

**Step 4 — Clone**  
Each repo card has a **Clone** button. Click it — the repo clones into `~/projects/<name>`.  
Already-cloned repos show an **Open** button that launches a terminal there.

**Filter / search:** use the All / GitHub / Bitbucket tabs and the search box.

---

### 4 — SSH Keys

Generate or register SSH keys for GitHub, Bitbucket, or any server.

| Field | Notes |
|---|---|
| Email | Used as the key comment |
| Key name | Saved as `~/.ssh/<name>` |
| Key type | ed25519 (recommended), RSA-4096, ECDSA-521 |
| Passphrase | Optional — leave blank for no passphrase |

After generating, the key is auto-added to `ssh-agent`.  
Click **List Keys** to see all keys in `~/.ssh/`.

---

### 5 — Tools

#### PHP Versions
- Click **Scan** to detect installed PHP versions
- Click **Install** / **Remove** next to any version
- Requires the [ondrej/php PPA](https://deb.sury.org/) — if not installed, the error box shows exact commands to add it

#### Apache Modules
- Click **Scan** to check which modules are enabled
- Toggle **Enable** / **Disable** — reloads Apache automatically
- Common modules: `rewrite` (required by most frameworks), `ssl`, `headers`, `http2`

#### PHP Extensions
- Click **Scan** to check installed extensions for the active PHP version
- Click **Install** / **Remove** to manage via `apt`

#### Database CLI
Three buttons open a root MySQL/MariaDB shell in your system terminal:

| Button | Command | When to use |
|---|---|---|
| MySQL / MariaDB | `sudo mysql -u root -p` | Standard — prompts for root password |
| MariaDB (explicit) | `sudo mariadb -u root -p` | If you have both MySQL and MariaDB installed |
| MySQL (socket auth) | `sudo mysql -u root` | Ubuntu's default — no password needed if root uses unix_socket plugin |

The terminal stays open after the session ends so you can see any error output.

**Supported terminals:** gnome-terminal, xfce4-terminal, konsole, tilix, mate-terminal, lxterminal, xterm, x-terminal-emulator.

---

## The `~/projects/` Directory

```
~/projects/
├── index.php          ← DevPanel welcome page (served at http://localhost)
├── myapp/             ← your project (cloned from git or created manually)
│   ├── public/
│   │   └── index.php
│   └── ...
└── anotherprojekt/
```

Each subdirectory is automatically detected by `index.php` and shown on the welcome page with:
- Git / Composer / index.php badges
- A link to the matching vhost if one exists

---

## The `devpanel.conf` File

Location: `/etc/apache2/sites-available/devpanel.conf`

All virtual hosts created by DevPanel are written here. Example:

```apache
# DevPanel managed VirtualHosts

<VirtualHost *:80>
    ServerName myapp.local
    DocumentRoot /home/alice/projects/myapp/public
    <Directory /home/alice/projects/myapp/public>
        Options -Indexes +FollowSymLinks
        AllowOverride All
        Require all granted
    </Directory>
    ErrorLog ${APACHE_LOG_DIR}/myapp.local-error.log
    CustomLog ${APACHE_LOG_DIR}/myapp.local-access.log combined
</VirtualHost>
```

You can also edit this file manually — DevPanel will pick up changes when you click **Scan** in the VirtualHosts tab.

---

## Sudo / Password Handling

DevPanel only asks for your sudo password when an action requires it (service restart, vhost creation, apt install).

- The password is validated once with `sudo -S -v`  
- Optionally save it for the session (stored in memory, never written to disk)
- The green **sudo active** / yellow **sudo locked** indicator in the sidebar shows the current state
- Click **Clear sudo** to immediately forget the cached password

---

## Configuration File

`~/.config/devpanel/config.toml`

```toml
repos_root    = "/home/alice/projects"
devpanel_conf = "/etc/apache2/sites-available/devpanel.conf"
hosts_file    = "/etc/hosts"
```

Edit this file if you want to point DevPanel at a different projects directory or use a non-standard Apache config path.

---

## Troubleshooting

### Apache won't start after adding a vhost
```bash
sudo apache2ctl configtest    # shows syntax errors
sudo journalctl -u apache2 -n 30  # recent logs
```

### `http://myapp.local` doesn't resolve
Check that the entry is in `/etc/hosts`:
```bash
grep myapp.local /etc/hosts
# Should show: 127.0.0.1  myapp.local
```
If missing, delete and re-create the vhost in DevPanel.

### DB terminal button shows "No terminal emulator found"
Install one:
```bash
sudo apt-get install -y gnome-terminal   # GNOME / Ubuntu
sudo apt-get install -y xterm            # minimal, works everywhere
sudo apt-get install -y xfce4-terminal  # XFCE
```

### Repos tab shows no GitHub repos
Install the GitHub CLI:
```bash
curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg \
  | sudo dd of=/usr/share/keyrings/githubcli-archive-keyring.gpg
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/githubcli-archive-keyring.gpg] \
  https://cli.github.com/packages stable main" \
  | sudo tee /etc/apt/sources.list.d/github-cli.list
sudo apt update && sudo apt install gh
gh auth login    # follow prompts to authenticate
```

### PHP install fails (version not found)
The `ondrej/php` PPA is required. DevPanel shows a fix box with the exact commands — click **Get Text File** to save them, or run:
```bash
sudo add-apt-repository ppa:ondrej/php
sudo apt-get update
```

### Welcome page (`index.php`) shows no vhosts
The `devpanel.conf` may not be enabled:
```bash
sudo a2ensite devpanel.conf
sudo systemctl reload apache2
```

---

## Project Structure

```
devpanel/
├── src/
│   ├── main.rs              # App entry, message loop, async tasks
│   ├── sudo_prompt.rs       # Sudo modal and password handling
│   ├── theme.rs             # Dark color palette
│   └── tabs/
│       ├── dashboard.rs     # Status cards, quick links
│       ├── repos.rs         # Remote repo browser
│       ├── vhosts.rs        # VirtualHost manager
│       ├── ssh_keys.rs      # SSH key generation
│       └── tools.rs         # PHP, Apache modules, extensions, DB CLI
├── scripts/
│   ├── devpanel-setup.sh    # First-run system setup
│   └── postinst             # dpkg post-install hook
├── share/
│   └── index.php            # Apache welcome page for ~/projects/
├── Cargo.toml
└── README.md
```

---

## License

MIT — see [LICENSE](LICENSE).
