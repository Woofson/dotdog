# Dot Matrix 🤖

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-2.1.0-green.svg)](Cargo.toml)

> *"We'll have none of that mister! How far did he get? What'd he touch?"*  
> — **Dot Matrix**, *Spaceballs*

**Dot Matrix** is a project compositor, dotfile manager, and versioned backup engine for Linux, macOS, and Windows. It lets you organize, version, encrypt, and backup files scattered across your entire filesystem **without moving them or creating fragile symlink webs**.

Featuring a **NoteDog-inspired multi-pane TUI**, 12 embedded theme presets, Age per-file encryption, independent per-project Git repositories, and intelligent path remapping for effortless distro hopping.

---

## 🌟 Why Dot Matrix?

Traditional dotfile managers (like GNU Stow, bare git repositories in `$HOME`, or symlink managers) force you into rigid compromises:
- **Symlink Fragility**: Moving or renaming files breaks links; tools that replace files on save (atomic writes) sever symlinks.
- **Home Directory Pollution**: A bare git repo in `$HOME` makes untracked files messy and risks accidental commits of sensitive data.
- **Monolithic Commits**: All your dotfiles end up in one giant repository, making it impossible to keep public configs (e.g. Neovim, Tmux) separate from sensitive ones (e.g. SSH keys, work VPNs).

**Dot Matrix solves this with in-place composition**:
1. **Zero Symlinks**: Files stay exactly where your apps expect them (`~/.config/nvim`, `~/.ssh/config`, `/etc/hosts`).
2. **Isolated Project Repositories**: Each project gets its own independent `.git/` repository, remote URL, and history.
3. **Multi-Pane Visual Explorer**: Inspect live syntax-highlighted previews, diffs, and project statistics at a glance.
4. **Strong Age Encryption**: Sensitive files are encrypted at rest with [Age](https://age-encryption.org) using modern cryptographic primitives (ChaCha20-Poly1305 / scrypt).

---

## ✨ Key Features

- 📁 **In-Place File Tracking**: Track files wherever they live across your system without copying or symlinking.
- 📦 **Per-Project Isolation**: Group related files into logical projects (`nvim-config`, `workstation`, `ssh-keys`). Each project has its own Git history and remote.
- 🎨 **NoteDog Visual Design**: 2-pane Superfile/NoteDog layout with rounded borders, modal dialogs, and clean status indicators.
- 🌈 **12 Embedded Themes**: Seamless support for `notedog`, `nord`, `catppuccin-mocha`, `dracula`, `gruvbox`, `tokyo-night`, `ayu-dark`, `solarized-dark`, `monokai`, `rose-pine`, `everforest`, and `one-dark`.
- 🪟 **Transparent Terminal Support**: Works natively with transparent and blurred terminal backdrops (`transparent_background = true`).
- 🔒 **Per-File Age Encryption**: Protect passwords, tokens, and private keys with passphrase-authenticated Age encryption.
- ⚡ **Three Track Modes**:
  - `[G] Git`: Version controlled, diffable, ready to push to GitHub/GitLab.
  - `[B] Backup`: Incremental deduplicated snapshots in local content-addressed storage.
  - `[+] Both`: Full version control plus local snapshot backup.
- 🔍 **Live Inspector & File Viewer**: Real-time syntax-highlighted inspector, full-file modal viewer, and fullscreen mode (`f` / `F11`).
- 📂 **Modular `conf.d` Assembly**: View split configuration directories (e.g. `sway/config.d/`, `fish/conf.d/`) assembled into a continuous annotated document.
- 🔄 **Distro & User Path Remapping**: Restore backups seamlessly across different operating systems or usernames (`/home/olduser` → `/home/newuser`).
- 💻 **Unified CLI & TUI**: Manage everything interactively via `dmxtui` or automate with `dmxcli` (including `--json` for scripts).

---

## 🚀 Installation

### Building from Source

Ensure you have Rust and Cargo installed ([rustup.rs](https://rustup.rs)):

```bash
git clone https://github.com/Woofson/dotmatrix.git
cd dotmatrix
cargo build --release
```

Install the binaries to your local path:

```bash
cp target/release/dmxtui ~/.local/bin/   # Terminal UI
cp target/release/dmxcli ~/.local/bin/   # CLI Tool
```

Or install directly via Cargo:

```bash
cargo install --path crates/dmtui
cargo install --path crates/dmcli
```

---

## 🎮 Quick Start Guide

### 1. Launch the TUI
```bash
dmxtui
```

### 2. Basic Workflow in 30 Seconds
1. **Create a Project**: Press `n` to create a new project (e.g., `nvim-config`).
2. **Add Files**: Press `Tab` or `2` to switch to the **Add Files** explorer.
   - Navigate with `j`/`k` or arrow keys.
   - Press `Enter` to step into folders, or `a` / `Enter` on a file to track it.
   - Press `t` to toggle track mode (`Git`, `Backup`, or `Both`).
   - Press `R` on a directory for recursive batch scanning.
3. **Inspect & Backup**: Press `Tab` or `1` to return to **Projects**.
   - Review your tracked files and live syntax-highlighted preview on the right pane.
   - Press `a` to create a backup commit with a custom message (or `A` for instant silent backup).
   - Press `G` to configure a Git remote and `p` to push to GitHub.

---

## 🖥️ TUI Multi-Pane Interface

Dot Matrix features a 2-pane NoteDog layout across three dedicated tabs:

### 1. 📦 Projects View (`Tab` / `1`)
- **Left Pane (42%)**: Collapsible tree of tracked projects and files with real-time status badges (`✓` Synced, `⚠` Drifted, `+` New, `✗` Missing) and encryption locks (`🔒`).
- **Right Pane (58%)**: 
  - **Project Selected**: Live dashboard showing Git remote state, sync health, file breakdown by track mode, and recent commit log.
  - **File Selected**: Live syntax-highlighted file inspector showing file size, path, and file contents.

### 2. 📂 Add Files Explorer (`Tab` / `2`)
- **Left Pane (52%)**: Fast directory navigator showing folders (`📁`), files (`📄`), tracked project indicators (`✓ [nvim-config]`), and file sizes.
- **Right Pane (48%)**: Target project status card, active add mode (`[G]`, `[B]`, `[+]`), quick action guide, and live preview of the currently highlighted file.

### 3. 🔄 Restore & Version Diff (`Tab` / `3`)
- **Left Pane (42%)**: Backup projects on disk and searchable commit revision history with timestamps and short hashes.
- **Right Pane (58%)**: Files contained in the selected commit (`NEW`, `CHG`, `OK`), multi-select checkboxes (`[*]`), backup previews (`b`), local file previews (`l`), and side-by-side diff inspection (`d`).

---

## ⌨️ Keyboard Shortcuts Reference

### Global Controls
| Key | Action |
|:---|:---|
| `Tab` / `Shift+Tab` | Cycle between tabs (Projects → Add Files → Restore) |
| `1`, `2`, `3` | Jump directly to Projects, Add Files, or Restore tab |
| `?` | Toggle Help Cheat Sheet modal |
| `!` / `F2` / `Ctrl+A` | Open About Dot Matrix modal (author & version info) |
| `f` / `F11` | Toggle Fullscreen viewer mode |
| `v` | Open full syntax-highlighted file viewer |
| `q` / `Esc` | Close dialog / Quit Dot Matrix |

### Projects Tab
| Key | Action |
|:---|:---|
| `j` / `↓`, `k` / `↑` | Navigate items |
| `PgUp` / `PgDn` | Page up / down (10 items) |
| `Home` / `End` | Jump to top / bottom |
| `Enter` / `l` / `→` | Expand / collapse project tree |
| `h` / `←` | Collapse project |
| `a` | Create backup commit with custom message prompt |
| `A` | Instant silent backup commit |
| `b` | Create standalone archive snapshot (`.tar.gz`, `.zip`, `.7z`) |
| `s` | Sync project hashes against disk |
| `x` | Toggle Age encryption for highlighted file |
| `X` | Toggle Age encryption for all files in highlighted project |
| `m` | Cycle track mode (`Git` → `Backup` → `Both`) |
| `n` / `Ctrl+N` | Create new project |
| `d` / `D` / `Ctrl+D` | Delete selected project (with confirmation) |
| `c` | Clean up deleted/missing files from manifest |
| `C` | Acknowledge missing files (mutes warnings) |
| `G` | Configure Git remote URL |
| `g` | Refresh Git remote sync status |
| `p` / `P` | Git Push / Git Pull from remote |
| `r` | Refresh projects and disk state |

### Add Files Tab
| Key | Action |
|:---|:---|
| `Enter` / `l` / `→` | Enter directory / Add file to target project |
| `h` / `←` / `Backspace` | Navigate to parent directory |
| `a` | Add selected file to target project |
| `R` | Open Recursive Scan modal (batch add files) |
| `t` | Cycle default add track mode (`[G]` → `[B]` → `[+]`) |
| `p` | Cycle target project |
| `n` | Create new project |
| `u` | Untrack selected file |
| `~` | Jump directly to user `$HOME` directory |

### Restore Tab
| Key | Action |
|:---|:---|
| `Enter` | Browse commits / Confirm file restore |
| `Space` | Toggle multi-select checkbox on file |
| `a` / `d` | Select all files / Deselect all |
| `b` | View file content from backup |
| `l` | View file content currently on local disk |
| `d` | View colorized line diff between backup and local |
| `h` / `←` / `Backspace` | Go back to commits / projects list |
| `r` | Refresh backup snapshots |

---

## 🎨 Themes & Customization

Dot Matrix includes 12 NoteDog-compatible themes embedded directly into the binary.

### Built-in Presets
| Preset | Style Description |
|:---|:---|
| `notedog` *(Default)* | Classic warm terminal palette with golden yellow and cyan accents |
| `nord` | Arctic cool blue and frost palette |
| `catppuccin-mocha` | Soothing modern pastel dark theme |
| `dracula` | Vibrant purple, pink, and cyan high-contrast theme |
| `gruvbox` | Retro groove warm brown and earth tones |
| `tokyo-night` | Deep indigo and neon night aesthetic |
| `ayu-dark` | Warm dark theme with crisp orange highlights |
| `solarized-dark` | Precision low-contrast teal and yellow palette |
| `monokai` | Iconic high-contrast hacker palette |
| `rose-pine` | Rosy pine, muted lilac, and gold palette |
| `everforest` | Calming natural green and moss aesthetic |
| `one-dark` | Clean Atom / VSCode dark classic |

### Configuration File (`~/.config/dotmatrix/config.toml`)
```toml
# Active theme name (matches preset or custom TOML filename in themes/)
theme = "catppuccin-mocha"

# Enable true transparent terminal background
transparent_background = false

# Show bottom status & shortcut hint bar
show_help_bar = true

# Auto-populate built-in themes into ~/.config/dotmatrix/themes/
spawn_themes = true
```

### Custom Themes
You can create your own theme by dropping a `.toml` file in `~/.config/dotmatrix/themes/my-theme.toml`:

```toml
name = "my-theme"
author = "Your Name"

[colors]
foreground = "#c0caf5"
background = "#1a1b26" # or "none" for transparent
active_border = "#7aa2f7"
inactive_border = "#414868"
sidebar_title = "#7dcfff"
active_sidebar_border = "#bb9af7"
highlight_bg = "#28345a"
highlight_fg = "#ffffff"
encrypted_tag = "#f7768e"

[palette]
primary = "#7aa2f7"
secondary = "#7dcfff"
accent = "#bb9af7"
border = "#414868"
```

---

## 🛠️ CLI Reference (`dmxcli`)

All Dot Matrix operations can be executed headlessly for scripting and automation:

```bash
# Project Management
dmxcli init                                    # Initialize Dot Matrix storage
dmxcli new <project> [-d "description"]        # Create project
dmxcli delete <project> [--force]              # Delete project
dmxcli list [-v]                               # List all tracked projects
dmxcli info <project>                          # Detailed project inspection

# File Operations
dmxcli add <project> <files...> [-t git|backup|both] [-e]  # Track files (optionally encrypted)
dmxcli remove <project> <files...>             # Stop tracking files
dmxcli status [project] [-c|--changes]         # Show drift status across files
dmxcli sync [project]                          # Synchronize file hashes

# Backups & Restores
dmxcli backup [project] [-m "Commit message"]  # Create backup commit
dmxcli backup [project] --archive [--format tar-gz|zip|7z]  # Create standalone archive
dmxcli restore <project> [files...] [--dry-run]             # Restore files from backup

# Git Remote Management
dmxcli git <project> remote [--set <url>]      # Get/Set Git remote URL
dmxcli git <project> push                      # Push project to Git remote
dmxcli git <project> pull                      # Pull project from Git remote
dmxcli git <project> log [-c 10]               # View commit history

# Global Scripting Flag
dmxcli list --json                             # Output machine-readable JSON
```

---

## 📂 Storage Architecture

Dot Matrix keeps all manifests in standard XDG configuration paths and stores project repositories in XDG data directories:

```
~/.config/dotmatrix/
├── config.toml           # General settings (theme, transparency, help bar)
├── manifest.toml         # User projects and tracked file definitions
└── themes/               # Custom and exported TOML themes

~/.local/share/dotmatrix/
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
cargo build --release          # Build optimized binaries
```

---

## 📄 License

This project is licensed under the **MIT License** — see the [LICENSE](LICENSE) file for details.

## 👤 Author

**Bolt J Woofson** — [@Woofson](https://github.com/Woofson)
