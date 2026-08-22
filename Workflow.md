# DotDog Workflow & Usage Scenarios 📘

This guide provides practical, end-to-end scenarios for managing, versioning, encrypting, and restoring your configurations with **DotDog** (formerly Dot Matrix).

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
│                               DOTDOG COMPOSITOR                                 │
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
Track your local Neovim, Tmux, and Starship configurations without symlinking them.

### Step-by-Step
1. Launch the interactive TUI:
   ```bash
   dotdog
   ```
2. Create a new project:
   - Press `n` → Enter project name `nvim-dots` → Press `Enter`.
3. Add your files:
   - Press `2` or `+` to open the embedded **File Explorer** in the right pane.
   - Navigate to `~/.config/nvim/`.
   - Highlight `init.lua` and press `a` (or `Enter`) to add it to `nvim-dots`.
   - Highlight `lua/plugins.lua` and press `a`.
   - Press `Esc` or `1` to return to the **Inspector View**.
4. Create your first backup commit:
   - Press `b` → Type commit message: `Initial Neovim configuration import` → Press `Enter`.
5. Edit your Neovim files normally in your favorite editor. When you reopen DotDog, changed files show a `⚠ Drifted` badge. Press `b` to commit updates.

---

## Scenario 2: Secure Secret Management with Age Encryption

### Goal
Track private SSH keys and sensitive tokens with passphrase-protected Age encryption.

### Step-by-Step
1. In DotDog, press `n` to create a new project called `ssh-vault`.
2. Press `+` to open File Explorer → Navigate to `~/.ssh/` → Highlight `id_ed25519` and press `a`.
3. In the sidebar, select `id_ed25519` and press `e` to toggle Age encryption (`🔒`).
4. Press `b` to commit the backup:
   - DotDog prompts for your encryption passphrase.
   - The private key is encrypted with Age (ChaCha20-Poly1305 / scrypt) before storage in the content-addressed store.
5. In Git, only the encrypted ciphertext is recorded, ensuring zero plaintext leakage even in remote repositories.

---

## Scenario 3: Multi-Project Composition with Independent Git Remotes

### Goal
Push public developer configs to GitHub while keeping personal workstation configs on GitLab.

### Step-by-Step
1. Select project `nvim-dots` in the top sidebar.
2. Press `G` → Enter remote URL: `git@github.com:youruser/nvim-dots.git` → Press `Enter`.
3. Press `p` to push your commits to GitHub.
4. Select project `workstation-dots` in the top sidebar.
5. Press `G` → Enter remote URL: `git@gitlab.com:youruser/private-workstation.git` → Press `Enter`.
6. Press `p` to push to GitLab.
7. Both repositories operate completely independently without colliding or sharing commit trees.

---

## Scenario 4: Machine Migration & Distro Hopping (Automatic Path Remapping)

### Goal
Restore your complete development setup on a brand-new laptop with a different username (`/home/alice` → `/home/bob`).

### Step-by-Step
1. On your new machine, clone your projects into DotDog data directory or initialize with:
   ```bash
   dotdog init
   ```
2. Open DotDog (`dotdog`) and select project `nvim-dots`.
3. Press `3` or `d` to open **Revisions & Restore View**.
4. Select the commit you wish to restore, mark all files with `a` (or `Space` for individual files), and press `Enter`.
5. DotDog automatically detects your current `$HOME` path and remaps `~/.config/...` cleanly to the new username.

---

## Scenario 5: Modular `conf.d` Assembly & Inspection

### Goal
Inspect a split configuration directory (e.g. `sway/config.d/`) assembled into a continuous document.

### Step-by-Step
1. Add modular snippet files (`~/.config/sway/config.d/01_input`, `02_keybinds`, `03_bar`) to project `sway-wm`.
2. In DotDog's **Inspector View (`1`)**, select the snippet files to view the synthesized live preview with syntax highlighting and line number gutters.
3. Press `f` or `F11` to toggle Fullscreen inspection mode.

---

## Scenario 6: Automated Nightly Backups & Headless CLI Scripting

### Goal
Automatically back up all modified dotfiles on a daily schedule using `cron` or a `systemd` user timer.

### Step 1: Create Backup Script (`~/.local/bin/dotdog-nightly.sh`)
```bash
#!/usr/bin/env bash
set -euo pipefail

# Refresh status and create commits for all projects with changes
for proj in $(dotdog list projects --json | jq -r '.projects[].name'); do
    # Check if there are changes
    if dotdog status "$proj" --json | jq -e '.projects[0].drifted > 0 or .projects[0].new > 0' > /dev/null; then
        echo "Backing up project: $proj"
        dotdog backup "$proj" -m "Automated nightly snapshot $(date +'%Y-%m-%d %H:%M')"
        dotdog backup "$proj" --archive -f targz
    fi
done

echo "DotDog nightly backup completed successfully."
```

Make it executable:
```bash
chmod +x ~/.local/bin/dotdog-nightly.sh
```

### Step 2: Systemd User Timer (`~/.config/systemd/user/dotdog.timer`)
```ini
[Unit]
Description=Nightly DotDog Backup Timer

[Timer]
OnCalendar=*-*-* 03:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

Enable the timer:
```bash
systemctl --user daemon-reload
systemctl --user enable --now dotdog.timer
```

---

## Scenario 7: NoteDog Theme Customization & Transparent Terminal Styling

### Goal
Customize the visual palette to match your desktop environment or activate a transparent terminal backdrop.

### Step 1: Enable Transparent Terminal Background
Edit `~/.config/dotdog/config.toml`:
```toml
theme = "dotdog"
transparent_background = true
show_help_bar = true
```

When `transparent_background = true`, DotDog bypasses background color fills so your terminal emulator's background opacity, blur, or wallpaper shines through cleanly.

### Step 2: Selecting Starter Themes
DotDog comes with built-in presets: `notedog`, `dotdog` (neon cyber matrix), `nord`, `catppuccin-mocha`, `catppuccin-latte`, `catppuccin-macchiato`, `catppuccin-frappe`, `dracula`, `gruvbox`, `tokyo-night`, `ayu-dark`, `ayu-mirage`, `solarized-dark`, `monokai`, `rose-pine`, `everforest`, `kanagawa`.

Launch `dotdog` to immediately enjoy your custom theme!
```bash
dotdog
```
