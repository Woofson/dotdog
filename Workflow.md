# Dot Matrix Workflow & Usage Scenarios 📘

This guide provides practical, end-to-end scenarios for managing, versioning, encrypting, and restoring your configurations with **Dot Matrix**.

---

## 📑 Table of Contents
1. [Core Mental Model](#core-mental-model)
2. [Scenario 1: Daily Dotfile Management (In-Place Tracking)](#scenario-1-daily-dotfile-management-in-place-tracking)
3. [Scenario 2: Secure Secret Management with Age Encryption](#scenario-2-secure-secret-management-with-age-encryption)
4. [Scenario 3: Multi-Project Composition with Independent Git Remotes](#scenario-3-multi-project-composition-with-independent-git-remotes)
5. [Scenario 4: Machine Migration & Distro Hopping (Automatic Path Remapping)](#scenario-4-machine-migration--distro-hopping-automatic-path-remapping)
6. [Scenario 5: Modular `conf.d` Assembly & Inspection](#scenario-5-modular-confd-assembly--inspection)
7. [Scenario 6: Automated Nightly Backups & Headless CLI Scripting](#scenario-6-automated-nightly-backups--headless-cli-scripting)
8. [Scenario 7: NoteDog Theme Customization & Transparent Terminal Styling](#scenario-7-notedog-theme-customization--transparent-terminal-styling)

---

## Core Mental Model

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                                 YOUR SYSTEM                                     │
│  ~/.config/nvim/init.lua      ~/.ssh/config          /etc/hosts                 │
└───────────────┬─────────────────────┬─────────────────────┬─────────────────────┘
                │ (tracked in-place)  │ (tracked in-place)  │ (tracked in-place)
                ▼                     ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              DOT MATRIX COMPOSITOR                              │
│                                                                                 │
│   📦 Project: nvim-dots       🔒 Project: ssh-vault       📦 Project: sys-admin │
│   ├─ .git/ (github.com/...)   ├─ .git/ (local encrypted)  ├─ .git/ (gitlab.com) │
│   ├─ store/ (blobs)           ├─ store/ (age encrypted)   ├─ store/ (blobs)     │
│   └─ index.json               └─ index.json               └─ index.json         │
└─────────────────────────────────────────────────────────────────────────────────┘
```

- **In-Place**: Files are never moved into a central dotfiles folder and never replaced with symlinks.
- **Projects**: Logical groups of files with independent Git histories and distinct remote destinations.
- **Track Modes**:
  - `[G] Git`: Tracked with Git version control; diffable and pushable.
  - `[B] Backup`: Tracked in incremental deduplicated storage.
  - `[+] Both`: Both version controlled and snapshot backed up.
- **Drift Engine**: Real-time SHA256 checking warns you when files have changed on disk (`⚠ Drifted`) or are newly tracked (`+ New`).

---

## Scenario 1: Daily Dotfile Management (In-Place Tracking)

### Goal
Track your Neovim and shell configurations without moving files or setting up symlinks.

### Step 1: Create the Project
In the TUI (`dmxtui`):
1. Press `n` on the **Projects** tab.
2. Enter the project name: `neovim-setup`.

*Or via CLI:*
```bash
dmxcli new neovim-setup -d "Neovim plugins and Lua configs"
```

### Step 2: Add Files
In the TUI:
1. Press `Tab` to switch to **Add Files** explorer.
2. Navigate to `~/.config/nvim/`.
3. Press `R` on the `nvim` folder to recursively scan all Lua and config files.
4. Review the detected files in the modal and press `Enter` to batch add them.

*Or via CLI:*
```bash
dmxcli add neovim-setup ~/.config/nvim/init.lua ~/.config/nvim/lua/
```

### Step 3: Work Normally in Neovim
Open and edit your files as you normally do:
```bash
nvim ~/.config/nvim/init.lua
```

### Step 4: Check Drift & Create Backup Commit
In the TUI:
1. Press `Tab` to return to the **Projects** tab.
2. You will see `init.lua` marked with `⚠` (Drifted) in yellow, and the right inspector pane displays the updated file preview.
3. Press `a` to create a backup commit.
4. Type a descriptive message: `Configure LSP keybindings and theme`.
5. Press `Enter`. All files are safely hashed and committed into the project repository.

*Or via CLI:*
```bash
dmxcli status neovim-setup
dmxcli backup neovim-setup -m "Configure LSP keybindings and theme"
```

---

## Scenario 2: Secure Secret Management with Age Encryption

### Goal
Track sensitive files (`~/.ssh/config`, `~/.ssh/id_ed25519.pub`, `.env` files) securely with passphrase-authenticated Age encryption.

### Step 1: Create a Secrets Project
```bash
dmxcli new ssh-vault -d "SSH client configuration and keys"
```

### Step 2: Add Files with Encryption
In the TUI:
1. Switch to **Add Files** (`Tab`).
2. Navigate to `~/.ssh/`.
3. Press `t` until the track mode shows `[+] Both` or `[B] Backup`.
4. Press `Enter` on `config` to track it.
5. Return to **Projects** (`Tab`). Highlight `~/.ssh/config` and press `x` to enable Age encryption. A `🔒` lock icon appears.

*Or via CLI:*
```bash
dmxcli add ssh-vault ~/.ssh/config -t both -e
```

### Step 3: Commit with Passphrase Protection
1. In the TUI, press `a` to create a backup commit for `ssh-vault`.
2. A password dialog appears: `Enter encryption passphrase for project: ssh-vault`.
3. Enter your secret passphrase and confirm.
4. Dot Matrix encrypts the files using Age before storing them. Plaintext secret contents **never enter unencrypted git objects**.

---

## Scenario 3: Multi-Project Composition with Independent Git Remotes

### Goal
Maintain a public Neovim config on GitHub while sending company-specific workstation configs to a private GitLab server.

### Architecture
- **Project `public-dots`** → `git@github.com:myuser/dotfiles.git`
- **Project `work-dots`** → `git@gitlab.company.com:team/work-station.git`

### Step 1: Configure Remotes
In the TUI:
1. Highlight `public-dots` on the Projects tab.
2. Press `G` to open the Git Remote configuration dialog.
3. Enter: `git@github.com:myuser/dotfiles.git` and press `Enter`.
4. Highlight `work-dots`, press `G`, and enter `git@gitlab.company.com:team/work-station.git`.

*Or via CLI:*
```bash
dmxcli git public-dots remote --set git@github.com:myuser/dotfiles.git
dmxcli git work-dots remote --set git@gitlab.company.com:team/work-station.git
```

### Step 2: Push Changes
1. Highlight the project on the left pane.
2. The right-hand dashboard displays your remote reachability and commits ahead/behind: `Ahead of remote (ready to push [p])`.
3. Press `p` to push to the remote repository.
4. Press `P` anytime to pull upstream changes.

*Or via CLI:*
```bash
dmxcli git public-dots push
dmxcli git work-dots push
```

---

## Scenario 4: Machine Migration & Distro Hopping (Automatic Path Remapping)

### Goal
Restore your complete development setup on a newly installed Linux machine or macOS laptop where the username or home path is different (`/home/alice` on Arch → `/home/bob` on Fedora or `/Users/bob` on macOS).

### Step 1: Transfer or Clone Backup Storage
Copy your `~/.local/share/dotmatrix/projects/` directory or clone your Git repositories onto the new machine.

### Step 2: Open Restore View
1. Launch `dmxtui` and press `Tab` or `3` to switch to **Restore & Diff**.
2. Select your backup project from the list on the left pane and press `Enter`.
3. Browse the commit history and press `Enter` on the revision snapshot you wish to inspect.

### Step 3: Inspect Diffs & Status
On the right pane, Dot Matrix automatically analyzes each file against your new machine's local disk:
- `NEW`: File does not exist yet on this machine.
- `CHG`: File exists locally but differs from the backup snapshot.
- `OK`: Local file matches the backup exactly.

Press:
- `d` to view a side-by-side colorized diff between the backup snapshot and local disk.
- `b` to view the backup copy.
- `l` to view the local copy.

### Step 4: Selective or Full Restore
1. Press `Space` to toggle individual files, or press `a` to select all files.
2. Press `Enter` to initiate restore.
3. The confirmation dialog will show automatic path translation:
   ```
   Original: /home/alice/.config/nvim/init.lua
   Remapped: /home/bob/.config/nvim/init.lua
   ```
4. Choose `[o] Original/Auto-Remapped Location` or press `[c]` to specify a custom target folder.
5. Press `y` to confirm. Your configuration is restored in place!

---

## Scenario 5: Modular `conf.d` Assembly & Inspection

### Goal
Inspect applications that split their settings across multiple files in a directory (such as `~/.config/fish/conf.d/` or `~/.config/sway/config.d/`).

### How It Works
Dot Matrix features built-in **conf.d directory assembly**:
1. When tracking a directory, files are sorted by numeric prefix first (`00-env.fish`, `10-aliases.fish`, `99-local.fish`), then alphabetically.
2. In the TUI, highlight the directory in **Projects** or **Add Files** and press `v`.
3. The file viewer concatenates all configuration snippets into a single continuous, syntax-highlighted document with clear visual divider banners between files.
4. Press `f` or `F11` to expand the viewer into fullscreen mode for reading.

---

## Scenario 6: Automated Nightly Backups & Headless CLI Scripting

### Goal
Run unattended nightly backups of all modified tracked dotfiles and export a compressed `.tar.gz` archive snapshot.

### Step 1: Create Backup Script (`~/.local/bin/dotmatrix-nightly.sh`)
```bash
#!/usr/bin/env bash
set -euo pipefail

# Refresh status and create commits for all projects with changes
for proj in $(dmxcli list --json | jq -r '.[].name'); do
    # Check if there are changes
    if dmxcli status "$proj" --json | jq -e '.summary.drifted > 0 or .summary.new > 0' > /dev/null; then
        echo "Backing up project: $proj"
        dmxcli backup "$proj" -m "Automated nightly snapshot $(date +'%Y-%m-%d %H:%M')"
        dmxcli backup "$proj" --archive --format tar-gz
    fi
done

echo "Dot Matrix nightly backup completed successfully."
```

Make it executable:
```bash
chmod +x ~/.local/bin/dotmatrix-nightly.sh
```

### Step 2: Systemd User Timer (`~/.config/systemd/user/dotmatrix.timer`)
```ini
[Unit]
Description=Nightly Dot Matrix Backup Timer

[Timer]
OnCalendar=*-*-* 03:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable the timer:
```bash
systemctl --user daemon-reload
systemctl --user enable --now dotmatrix.timer
```

---

## Scenario 7: NoteDog Theme Customization & Transparent Terminal Styling

### Goal
Customize the visual palette to match your desktop environment or activate a transparent terminal backdrop.

### Step 1: Enable Transparent Terminal Background
Edit `~/.config/dotmatrix/config.toml`:
```toml
theme = "nord"
transparent_background = true
show_help_bar = true
```

When `transparent_background = true`, Dot Matrix bypasses background color fills so your terminal emulator's background opacity, blur, or wallpaper shines through cleanly.

### Step 2: Creating a Custom Theme
Create `~/.config/dotmatrix/themes/cyberpunk.toml`:
```toml
name = "cyberpunk"
author = "Bolt J Woofson"

[colors]
foreground = "#00ff9f"
background = "#0b0e14"
active_border = "#ff007f"
inactive_border = "#2b213a"
sidebar_title = "#00f0ff"
active_sidebar_border = "#ffe600"
highlight_bg = "#3d155f"
highlight_fg = "#ffffff"
encrypted_tag = "#ff003c"

[palette]
primary = "#00f0ff"
secondary = "#ffe600"
accent = "#ff007f"
border = "#2b213a"
```

In `~/.config/dotmatrix/config.toml`, set:
```toml
theme = "cyberpunk"
```
Launch `dmxtui` to enjoy your new custom palette!
