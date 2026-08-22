# DotDog 🐶

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-3.0.1-green.svg)](Cargo.toml)

> *"We'll have none of that mister! How far did he get? What'd he touch?"*  
> — **Dot Matrix**, *Spaceballs*

> [!NOTE]
> **DotDog** was previously known as **Dot Matrix**. Developed by **Bolt J Woofson** ([@Woofson](https://github.com/Woofson)), **DotDog** is the dotfile & project compositor companion to [NoteDog](https://github.com/Woofson/notedog) in the Woofson canine terminal ecosystem.

**DotDog** is a project compositor, dotfile manager, and versioned backup engine for Linux, macOS, and Windows. It lets you organize, version, encrypt, and backup files scattered across your entire filesystem **without moving them or creating fragile symlink webs**.

Featuring a **NoteDog-inspired persistent multi-pane TUI**, 17+ embedded theme presets, Age per-file encryption, independent per-project Git repositories, dynamic icon matching, and intelligent path remapping for effortless machine migration.

---

## 🌟 Why DotDog?

Traditional dotfile managers (GNU Stow, bare git repositories in `$HOME`, or symlink managers) force you into rigid compromises:
- **Symlink Fragility**: Moving or renaming files breaks links; tools that replace files on save (atomic writes) sever symlinks.
- **Home Directory Pollution**: A bare git repo in `$HOME` makes untracked files messy and risks accidental commits of sensitive data.
- **Monolithic Commits**: All your dotfiles end up in one giant repository, making it impossible to keep public configs (e.g. Neovim, Tmux) separate from sensitive ones (e.g. SSH keys, work VPNs).

**DotDog solves this with in-place composition**:
1. **Zero Symlinks**: Files stay exactly where your apps expect them (`~/.config/nvim`, `~/.ssh/config`, `/etc/hosts`).
2. **Isolated Project Repositories**: Each project gets its own independent `.git/` repository, remote URL, and history.
3. **Persistent NoteDog Sidebar**: Inspect live syntax-highlighted previews, diffs, and project statistics at a glance.
4. **Strong Age Encryption**: Sensitive files are encrypted at rest with [Age](https://age-encryption.org) using modern cryptographic primitives (ChaCha20-Poly1305 / scrypt).
5. **Unified Single Binary**: Run `dotdog` for the full interactive TUI, or `dotdog <command>` for headless CLI scripting.

---

## ✨ Key Features

- 📁 **In-Place File Tracking**: Track files wherever they live across your system without copying or symlinking.
- 📦 **Per-Project Isolation**: Group related files into logical projects (`nvim-config`, `workstation`, `ssh-keys`). Each project has its own Git history and remote.
- 🎨 **NoteDog Visual Architecture**: 2-level persistent sidebar anchoring projects and files on the left with a dynamic workspace on the right.
- 🌈 **17+ Embedded Starter Themes**: `notedog`, `dotdog` (neon cyber matrix), `nord`, `catppuccin-mocha`, `catppuccin-latte`, `catppuccin-macchiato`, `catppuccin-frappe`, `dracula`, `gruvbox`, `tokyo-night`, `ayu-dark`, `ayu-mirage`, `solarized-dark`, `monokai`, `rose-pine`, `everforest`, and `kanagawa`.
- 🪟 **Transparent Terminal Support**: Works natively with transparent and blurred terminal backdrops (`transparent_background = true`).
- ⚡ **Dynamic Project Icons**: Match project names with regex rules to assign custom emoji/nerd icons (⚡ Neovim, 🐚 Shell, 🔑 Vault, 🐙 Git, 🖥️ Terminal, 🪟 Window Manager, 🐳 Docker, 🦀 Rust, 🐍 Python).
- 🔒 **Per-File Age Encryption**: Protect passwords, tokens, and private keys with passphrase-authenticated Age encryption.
- 🚦 **Three Track Modes**:
  - `[G] Git`: Version controlled, diffable, ready to push to GitHub/GitLab.
  - `[B] Backup`: Incremental deduplicated snapshots in local content-addressed storage.
  - `[+] Both`: Full version control plus local snapshot backup.
- 🔍 **Live Inspector & File Viewer**: Real-time syntax-highlighted inspector, full-file modal viewer, and fullscreen mode (`f` / `F11`).
- 🔄 **Distro & User Path Remapping**: Restore backups seamlessly across different operating systems or usernames (`/home/olduser` → `/home/newuser`).
- 💻 **Single Binary with CLI & TUI**: Manage interactively via `dotdog` or automate with `dotdog status`, `dotdog backup`, `dotdog sync` (with `--json` support).

---

## 🚀 Installation

### Arch Linux (AUR)
```bash
yay -S dotdog-git
```

### Building from Source

Ensure you have Rust and Cargo installed ([rustup.rs](https://rustup.rs)):

```bash
git clone https://github.com/Woofson/dotdog.git
cd dotdog
cargo build --release
```

Install the binary to your path:

```bash
cp target/release/dotdog ~/.local/bin/
```

Or install directly via Cargo:

```bash
cargo install --path crates/dotdog
```

*(Note: Both `dotdog` and `dmx` binaries are built and installed).*

---

## 🎮 Quick Start Guide

### 1. Launch the TUI
```bash
dotdog
```
*(or run `dmx`)*

### 2. Basic Workflow in 30 Seconds
1. **Create a Project**: Press `n` to create a new project (e.g., `nvim-config`).
2. **Add Files**: Press `+` or `2` to open the embedded **File Explorer** in the right pane.
   - Navigate with `j`/`k` or arrow keys.
   - Press `Enter` to step into folders, or `a` / `Enter` on a file to track it directly into the active sidebar project.
   - Press `t` to toggle track mode (`Git`, `Backup`, or `Both`).
   - Press `R` on a directory for recursive batch scanning.
   - Press `Esc` or `q` when done to return to the Inspector.
3. **Inspect & Backup**:
   - Navigate between **Projects** (top sidebar) and **Files** (bottom sidebar) with `Tab` or `j`/`k`.
   - Review live project health cards and syntax-highlighted file contents in the right workspace.
   - Press `b` to create a backup commit with a custom message (or `B` for instant silent backup).
   - Press `G` to configure a Git remote and `p` to push to GitHub.
   - Press `3` or `d` to view commit history, line diffs, and restore files.

---

## 🖥️ NoteDog Sidebar Interface Architecture

DotDog uses a persistent 2-level sidebar anchoring your projects and tracked files on the left, with a dynamic workspace on the right:

```
┌─────────────────────────┬────────────────────────────────────────────────────────┐
│ 📦 PROJECTS (Top Left)  │ 🔍 LIVE INSPECTOR & WORKSPACE (Right 66%)              │
│ ▶ ⚡ nvim-config   [✓][G]│                                                        │
│   🐚 shell-dots    [⚠][+]│ Git Remote: git@github.com:user/nvim-config.git        │
│   🔑 ssh-vault    [🔒][B]│ Sync Health: 4/4 Synced ✓ (Up to date with remote)      │
│   ⚙️ workstation   [✓][G]│                                                        │
├─────────────────────────┤ Recent Commits:                                        │
│ 📋 TRACKED FILES (Btm)  │ • 7a9f2c1 (2h ago) Add telescope and lualine configs   │
│ ✓ [G] init.lua          │ • 1b4e883 (Yesterday) Initial import                   │
│ ⚠ [G] lua/plugins.lua   │ ────────────────────────────────────────────────────── │
│ ✓ [G] lua/options.lua   │ 📄 LIVE PREVIEW: ~/.config/nvim/init.lua               │
│                         │  1 │ local opt = vim.opt                               │
│                         │  2 │ opt.relativenumber = true                         │
│                         │  3 │ opt.tabstop = 4                                   │
└─────────────────────────┴────────────────────────────────────────────────────────┘
```

### 1. 📦 Top Sidebar: Projects List
- Lists all repositories/projects with dynamic icons, real-time health badges (`✓` Synced, `⚠` Drifted, `+` New, `✗` Missing), primary track mode (`[G]`, `[B]`, `[+]`), and encryption tag (`🔒`).
- Switching the active project automatically updates the bottom tracked files list and the right workspace.

### 2. 📋 Bottom Sidebar: Tracked Files List
- Displays all files tracked within the currently active project with status indicators, track modes, file sizes, and encryption locks.

### 3. 🔍 Right Main Workspace
- **Inspector Mode (`1` / Default)**: Live project dashboard with Git sync health, remote URLs, recent commits, and live syntax-highlighted file preview with line number gutters.
- **Add Files Explorer (`2` / `+`)**: Directory navigator to browse disk and drop files directly into the active sidebar project.
- **Revisions & Diffs (`3` / `d`)**: Searchable commit history, file change badges (`NEW`, `CHG`, `OK`), line diff inspection, and multi-file restore controls.
- **Fullscreen Viewer (`f` / `F11`)**: Expands file or diff viewing to the entire terminal.

---

## ⌨️ Keyboard Shortcuts Reference

### Navigation & Focus
| Key | Action |
|:---|:---|
| `Tab` / `Shift+Tab` | Cycle focus across panes (**Projects** → **Files** → **Main Workspace**) |
| `h` / `l` or `←` / `→` | Move focus between Sidebar and Main Workspace |
| `j` / `k` or `↓` / `↑` | Move up / down items in the focused pane |
| `PgUp` / `PgDn` | Page up / down (10 items) |
| `Home` / `End` | Jump to top / bottom of focused pane |
| `1` / `i` | Return to Live Inspector |
| `2` / `+` / `a` | Open embedded File Explorer to add files to active project |
| `3` / `d` | Open Revisions, Diffs & Restore |
| `f` / `F11` | Toggle Fullscreen file / diff inspection |
| `v` / `Enter` | Open full syntax-highlighted viewer on selected file |
| `q` / `Esc` | Return to Inspector (or quit DotDog from top level) |

### Project & File Actions
| Key | Action |
|:---|:---|
| `b` | Create backup commit with custom message prompt |
| `B` (Shift+B) | Instant silent backup commit (timestamp message) |
| `Ctrl+B` | Create standalone archive snapshot (`.tar.gz`, `.zip`, `.7z`) |
| `s` | Sync project hashes against disk |
| `e` | Toggle Age encryption for active file |
| `E` (Shift+E) | Toggle Age encryption for all files in active project |
| `t` | Cycle track mode (`[G]` Git → `[B]` Backup → `[+]` Both) |
| `u` / `d` | Untrack selected file from project |
| `n` | Create new project modal |
| `D` | Delete selected project (with confirmation) |
| `c` | Clean up deleted/missing files from manifest |
| `C` | Acknowledge missing files (mutes warnings) |
| `G` | Configure Git remote repository URL |
| `g` | Refresh Git remote sync status |
| `p` / `P` | Git Push / Git Pull from remote repository |
| `r` | Refresh project status and disk state |
| `?` | Toggle Help Cheat Sheet modal |
| `!` / `F2` | Open About DotDog modal |

---

## 🎨 Themes & Customization

DotDog includes 17 NoteDog-compatible themes embedded directly into the binary.

### Built-in Presets
| Preset | Style Description |
|:---|:---|
| `notedog` *(Default)* | Classic warm terminal palette with golden amber and cyan accents |
| `dotdog` *(Dot Matrix)*| High-tech Cyberpunk neon matrix green and electric cyan |
| `nord` | Arctic cool blue and frost palette |
| `catppuccin-mocha` | Soothing modern pastel dark theme |
| `catppuccin-latte` | Crisp light pastel theme |
| `catppuccin-macchiato`| Medium contrast pastel dark theme |
| `catppuccin-frappe` | Muted dark pastel theme |
| `dracula` | Vibrant purple, pink, and cyan high-contrast theme |
| `gruvbox` | Retro groove warm brown and earth tones |
| `tokyo-night` | Deep indigo and neon night aesthetic |
| `ayu-dark` | Warm dark theme with crisp orange highlights |
| `ayu-mirage` | Deep slate-navy theme with warm accents |
| `solarized-dark` | Precision low-contrast teal and yellow palette |
| `monokai` | Iconic high-contrast hacker palette |
| `rose-pine` | Rosy pine, muted lilac, and gold palette |
| `everforest` | Calming natural green and moss aesthetic |
| `kanagawa` | Japanese ink wash painting aesthetic with soft hues |

### Configuration File (`~/.config/dotdog/config.toml`)
```toml
# Active theme name (matches preset or custom TOML filename in themes/)
theme = "notedog"

# Enable true transparent terminal background
transparent_background = true

# Show bottom status & shortcut hint bar
show_help_bar = true

# Auto-populate built-in themes into ~/.config/dotdog/themes/
spawn_themes = true

[icons]
project = "📦 "
file = "📄 "
folder = "📁 "
encrypted = "🔒 "

# Regex pattern rules for project icons
[[icons.rules]]
pattern = "(?i).*(nvim|neovim|vim).*"
icon = "⚡ "

[[icons.rules]]
pattern = "(?i).*(zsh|bash|fish|shell).*"
icon = "🐚 "

[[icons.rules]]
pattern = "(?i).*(ssh|vault|secret|key).*"
icon = "🔑 "

[layout]
sidebar_width = "34%"
projects_height = "45%"
files_height = "55%"
```

---

## 🛠️ CLI Reference (`dotdog <subcommand>`)

All DotDog operations can be executed headlessly for scripting and automation:

```bash
# Project Management
dotdog init                                        # Initialize DotDog storage
dotdog new <project> [-d "description"]            # Create project
dotdog list [projects|files|archives|commits]      # List tracked items
dotdog status [project] [-c|--changes]             # Show drift status across files
dotdog sync [project]                              # Synchronize file hashes

# File Operations
dotdog add <project> <files...> [-t git|backup|both] [-e]  # Track files (optionally encrypted)
dotdog remove <project> <files...>                 # Stop tracking files
dotdog clean <project>                             # Clean up deleted files from manifest
dotdog ack <project> [file]                        # Acknowledge missing files

# Backups & Restores
dotdog backup [project] [-m "Commit message"]      # Create backup commit
dotdog backup [project] --archive [-f targz|zip|7z]# Create standalone archive
dotdog restore <project> [files...] [-o /dest]     # Restore files from backup

# Git Remote Management
dotdog remote <project> [url]                      # Get/Set Git remote URL
dotdog push <project>                              # Push project to Git remote
dotdog pull <project>                              # Pull project from Git remote
dotdog verify [project]                            # Verify file integrity and hashes

# Global Scripting Flag
dotdog status --json                               # Output machine-readable JSON
```

---

## 📂 Storage Architecture

DotDog keeps all manifests in standard XDG configuration paths and stores project repositories in XDG data directories (with automatic backwards-compatible detection of `~/.config/dotmatrix/`):

```
~/.config/dotdog/
├── config.toml           # General settings (theme, transparency, help bar, icons)
├── manifest.toml         # User projects and tracked file definitions
└── themes/               # Custom and exported TOML themes

~/.local/share/dotdog/
├── projects/
│   ├── nvim-config/
│   │   ├── .git/         # Isolated Git repository for this project
│   │   ├── store/        # Content-addressed snapshot blobs
│   │   └── index.json    # File tracking SHA256 index
│   └── ssh-keys/
│       ├── .git/
│       ├── store/
│       └── index.json
└── backups/              # Standalone archive backups (.tar.gz, .zip, .7z)
```

---

## 🤝 Contributing & Development

We welcome contributions! To build and test the project locally:

```bash
cargo check --workspace        # Verify code sanity
cargo test --workspace         # Run test suites
cargo build --release          # Build optimized binary
```

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Bolt J Woofson** — [@Woofson](https://github.com/Woofson)
