# DevPanel — Packaging & Distribution Guide

## Overview

This guide covers everything you need to go from source code to a distributable
`.deb` package that users can install on Debian/Ubuntu.

---

## Prerequisites

Make sure you have Rust and Cargo installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Also install the GTK development library (required by the file dialog):

```bash
sudo apt install libgtk-3-dev
```

---

## Step 1 — Write `devpanel.desktop`

Create this file in your project root:

```ini
[Desktop Entry]
Name=DevPanel
Comment=Local development environment manager
Exec=/usr/bin/devpanel
Icon=devpanel
Terminal=false
Type=Application
Categories=Development;System;
Keywords=apache;mysql;php;ssh;
StartupWMClass=devpanel
```

> **Note:** `Exec=` points to `/usr/bin/devpanel` — this is where the `.deb`
> will install the binary. Do **not** use `target/release/devpanel` here.

---

## Step 2 — Write a `LICENSE` file

`cargo-deb` expects a license file to exist. Create `LICENSE` in your project root:

```
MIT License

Copyright (c) 2025 Your Name

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
```

---

## Step 3 — Update `Cargo.toml`

Add author info and the `[package.metadata.deb]` section:

```toml
[package]
name = "devpanel"
version = "0.1.0"
edition = "2021"
description = "Local development environment manager for Debian/Ubuntu"
authors = ["Your Name <you@email.com>"]

[package.metadata.deb]
maintainer = "Your Name <you@email.com>"
copyright = "2025 Your Name"
license-file = ["LICENSE"]
extended-description = """
DevPanel is a desktop GUI for managing your local development environment.
Control Apache, MySQL and PHP services, create Apache VirtualHosts,
and manage SSH keys — all from a single polished interface.
"""
depends = "$auto"
section = "devel"
priority = "optional"
assets = [
  ["target/release/devpanel", "usr/bin/",                       "755"],
  ["icon.png",                "usr/share/pixmaps/devpanel.png", "644"],
  ["devpanel.desktop",        "usr/share/applications/",        "644"],
]

[dependencies]
iced = { version = "0.13", features = ["tokio", "image"] }
tokio = { version = "1", features = ["full"] }
rfd = "0.15"
image = "0.25"
```

---

## Step 4 — Install `cargo-deb`

This is a one-time setup:

```bash
cargo install cargo-deb
```

---

## Step 5 — Build the `.deb`

```bash
cargo deb
```

This does everything in one command:
- Compiles the release binary (`cargo build --release`)
- Bundles the binary, icon, and `.desktop` file
- Produces the `.deb` at:

```
target/debian/devpanel_0.1.0_amd64.deb
```

---

## Step 6 — Test locally before distributing

```bash
# Install
sudo dpkg -i target/debian/devpanel_0.1.0_amd64.deb

# Verify it launches
devpanel

# Verify it appears in the app launcher
# (log out and back in if it doesn't appear immediately)

# Uninstall cleanly when done testing
sudo apt remove devpanel
```

---

## Step 7 — Distribute

Choose one of the following options depending on your audience:

---

### Option A — GitHub Releases (simplest, recommended to start)

1. Push your project to GitHub
2. Go to your repo → **Releases** → **Draft a new release**
3. Set a version tag (e.g. `v0.1.0`)
4. Upload `target/debian/devpanel_0.1.0_amd64.deb` as a release asset
5. Publish the release

Users install it with:

```bash
# Download from GitHub releases page, then:
sudo dpkg -i devpanel_0.1.0_amd64.deb

# Or fix any missing dependencies automatically:
sudo apt install -f
```

---

### Option B — Launchpad PPA (apt installable, best for Ubuntu users)

Users can install directly via `apt` — no manual download needed.

1. Create an account at [launchpad.net](https://launchpad.net)
2. Create a PPA (Personal Package Archive) from your profile page
3. Set up GPG signing keys and upload your source package
4. Once approved, users install with:

```bash
sudo add-apt-repository ppa:yourname/devpanel
sudo apt update
sudo apt install devpanel
```

> This option requires more setup (GPG keys, source packages) but gives
> users the cleanest install experience with automatic updates via `apt upgrade`.

---

### Option C — Flatpak / Flathub (distro-agnostic)

Works on any Linux distro, not just Debian/Ubuntu.

1. Write a Flatpak manifest `org.yourname.DevPanel.yml`
2. Test locally with `flatpak-builder`
3. Submit to [flathub.org](https://flathub.org) via a pull request to their GitHub repo
4. Once merged, users install with:

```bash
flatpak install flathub org.yourname.DevPanel
```

---

## Full Footstep Order

```
1.  Create devpanel.desktop in project root
2.  Create LICENSE in project root
3.  Update Cargo.toml with metadata and [package.metadata.deb]
4.  cargo install cargo-deb          (one time only)
5.  cargo deb                        (builds + packages everything)
6.  sudo dpkg -i target/debian/devpanel_*.deb   (test install)
7.  devpanel                         (test it runs)
8.  sudo apt remove devpanel         (test uninstall)
9.  Upload .deb to GitHub Releases / PPA / Flathub
```

---

## Versioning

When you release a new version, update the version in `Cargo.toml`:

```toml
[package]
version = "0.2.0"
```

Then rebuild:

```bash
cargo deb
# Produces: target/debian/devpanel_0.2.0_amd64.deb
```

Users who installed via `dpkg` can upgrade by installing the new `.deb` over the old one:

```bash
sudo dpkg -i devpanel_0.2.0_amd64.deb
```

Users who installed via a PPA will get the update automatically via `apt upgrade`.

---

## Project File Structure (for reference)

```
devpanel/
├── Cargo.toml               ← includes [package.metadata.deb]
├── LICENSE                  ← required by cargo-deb
├── icon.png                 ← app icon (512x512, embedded + installed)
├── devpanel.desktop         ← app launcher entry
└── src/
    ├── main.rs
    ├── sudo_prompt.rs
    ├── theme.rs
    └── tabs/
        ├── mod.rs
        ├── dashboard.rs
        ├── apache_touch.rs
        └── ssh_keys.rs
```
