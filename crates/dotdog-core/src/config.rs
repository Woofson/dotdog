//! Global configuration for dotmatrix
//!
//! Handles global settings that apply across all projects.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Preferred interface when running without arguments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PreferredInterface {
    /// Platform default (GUI on Windows, TUI on Linux/macOS)
    #[default]
    Auto,
    /// Always use GUI
    Gui,
    /// Always use TUI
    Tui,
}

/// Global dotmatrix configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Custom data directory path (optional, defaults to system data dir)
    #[serde(default)]
    pub data_dir: Option<String>,

    /// Default backup mode for new files
    #[serde(default)]
    pub default_backup_mode: BackupMode,

    /// Default archive format
    #[serde(default)]
    pub default_archive_format: ArchiveFormat,

    /// Enable git tracking by default
    #[serde(default = "default_true")]
    pub git_enabled: bool,

    /// Global exclude patterns
    #[serde(default = "default_excludes")]
    pub exclude: Vec<String>,

    /// Preferred interface when running without arguments
    #[serde(default)]
    pub preferred_interface: PreferredInterface,

    /// Owner name shown in About dialog (optional)
    #[serde(default)]
    pub owner_name: Option<String>,

    /// Owner website shown in About dialog (optional)
    #[serde(default)]
    pub owner_website: Option<String>,

    /// Owner email shown in About dialog (optional)
    #[serde(default)]
    pub owner_email: Option<String>,

    /// Theme name
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Transparent background (true = use terminal default bg)
    #[serde(default = "default_true")]
    pub transparent_background: bool,

    /// Show bottom status and help bar
    #[serde(default = "default_true")]
    pub show_help_bar: bool,

    /// Auto-populate theme files in ~/.config/dotmatrix/themes/
    #[serde(default = "default_true")]
    pub spawn_themes: bool,

    /// Icon customizations for projects and files
    #[serde(default)]
    pub icons: IconConfig,

    /// Visual layout sizing
    #[serde(default)]
    pub layout: LayoutConfig,
}

fn default_project_icon() -> String { "📦 ".to_string() }
fn default_file_icon() -> String { "📄 ".to_string() }
fn default_folder_icon() -> String { "📁 ".to_string() }
fn default_encrypted_icon() -> String { "🔒 ".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconRule {
    pub pattern: String,
    pub icon: String,
    #[serde(default)]
    pub target: Option<String>,
}

impl IconRule {
    pub fn matches(&self, name: &str) -> bool {
        if let Ok(re) = regex::Regex::new(&self.pattern) {
            if re.is_match(name) {
                return true;
            }
        } else if name.to_lowercase().contains(&self.pattern.to_lowercase()) {
            return true;
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconConfig {
    #[serde(default = "default_project_icon")]
    pub project: String,
    #[serde(default = "default_file_icon")]
    pub file: String,
    #[serde(default = "default_folder_icon")]
    pub folder: String,
    #[serde(default = "default_encrypted_icon")]
    pub encrypted: String,
    #[serde(default = "default_icon_rules")]
    pub rules: Vec<IconRule>,
}

fn default_icon_rules() -> Vec<IconRule> {
    vec![
        IconRule { pattern: "(?i).*(nvim|neovim|vim).*".to_string(), icon: "⚡ ".to_string(), target: None },
        IconRule { pattern: "(?i).*(zsh|bash|fish|shell|sh|shrc).*".to_string(), icon: "🐚 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(ssh|vault|secret|gpg|auth|key|password).*".to_string(), icon: "🔑 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(git|github|gitlab).*".to_string(), icon: "🐙 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(tmux|wezterm|alacritty|kitty|foot|term|ghostty).*".to_string(), icon: "🖥️ ".to_string(), target: None },
        IconRule { pattern: "(?i).*(hypr|sway|i3|bspwm|waybar|wm|desktop|polybar).*".to_string(), icon: "🪟 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(docker|container|podman|compose).*".to_string(), icon: "🐳 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(python|py).*".to_string(), icon: "🐍 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(rust|cargo).*".to_string(), icon: "🦀 ".to_string(), target: None },
        IconRule { pattern: "(?i).*(dot|dotfile|dots|config|workstation|system).*".to_string(), icon: "⚙️ ".to_string(), target: None },
    ]
}

impl Default for IconConfig {
    fn default() -> Self {
        Self {
            project: default_project_icon(),
            file: default_file_icon(),
            folder: default_folder_icon(),
            encrypted: default_encrypted_icon(),
            rules: default_icon_rules(),
        }
    }
}

impl IconConfig {
    pub fn get_project_icon(&self, name: &str) -> String {
        for rule in &self.rules {
            if rule.matches(name) {
                return rule.icon.clone();
            }
        }
        self.project.clone()
    }
}

fn default_sidebar_width() -> String { "34%".to_string() }
fn default_projects_height() -> String { "45%".to_string() }
fn default_files_height() -> String { "55%".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutConfig {
    #[serde(default = "default_sidebar_width")]
    pub sidebar_width: String,
    #[serde(default = "default_projects_height")]
    pub projects_height: String,
    #[serde(default = "default_files_height")]
    pub files_height: String,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            sidebar_width: default_sidebar_width(),
            projects_height: default_projects_height(),
            files_height: default_files_height(),
        }
    }
}

pub fn parse_constraint(s: &str, default_pct: u16) -> u16 {
    let trimmed = s.trim();
    if trimmed.ends_with('%') {
        if let Ok(pct) = trimmed[..trimmed.len() - 1].parse::<u16>() {
            return pct;
        }
    }
    default_pct
}

fn default_theme() -> String {
    "notedog".to_string()
}

fn default_true() -> bool {
    true
}

fn default_excludes() -> Vec<String> {
    vec![
        "**/*.log".to_string(),
        "**/.DS_Store".to_string(),
        "**/node_modules/**".to_string(),
        "**/.git/**".to_string(),
        "**/target/**".to_string(),
    ]
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: None,
            default_backup_mode: BackupMode::default(),
            default_archive_format: ArchiveFormat::default(),
            git_enabled: true,
            exclude: default_excludes(),
            preferred_interface: PreferredInterface::default(),
            owner_name: None,
            owner_website: None,
            owner_email: None,
            theme: default_theme(),
            transparent_background: true,
            show_help_bar: true,
            spawn_themes: true,
            icons: IconConfig::default(),
            layout: LayoutConfig::default(),
        }
    }
}

/// Backup mode for tracked files
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BackupMode {
    /// Content-addressed incremental backups
    #[default]
    Incremental,
    /// Archive-based backups (tarball)
    Archive,
}

impl BackupMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupMode::Incremental => "incremental",
            BackupMode::Archive => "archive",
        }
    }
}

/// Archive format for archive backups
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveFormat {
    TarGz,
    Zip,
    SevenZ,
}

impl Default for ArchiveFormat {
    fn default() -> Self {
        #[cfg(windows)]
        {
            ArchiveFormat::Zip
        }
        #[cfg(not(windows))]
        {
            ArchiveFormat::TarGz
        }
    }
}

impl ArchiveFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ArchiveFormat::TarGz => "tar.gz",
            ArchiveFormat::Zip => "zip",
            ArchiveFormat::SevenZ => "7z",
        }
    }
}

impl Config {
    /// Load config from the default location
    /// Creates the config file with defaults if it doesn't exist
    pub fn load() -> anyhow::Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let contents = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            // Create default config and save it
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save config to the default location
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(&path, contents)?;
        Ok(())
    }

    /// Get the config file path
    pub fn config_path() -> anyhow::Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Get the config directory path (~/.config/dotdog, with fallback to ~/.config/dotmatrix)
    pub fn config_dir() -> anyhow::Result<PathBuf> {
        let base = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        let dotdog_dir = base.join("dotdog");
        let dotmatrix_dir = base.join("dotmatrix");
        if dotdog_dir.exists() {
            Ok(dotdog_dir)
        } else if dotmatrix_dir.exists() {
            Ok(dotmatrix_dir)
        } else {
            Ok(dotdog_dir)
        }
    }

    /// Get the themes directory path (~/.config/dotdog/themes)
    pub fn themes_dir() -> anyhow::Result<PathBuf> {
        Ok(Self::config_dir()?.join("themes"))
    }

    /// Get the data directory path (where backups/store lives)
    pub fn data_dir(&self) -> anyhow::Result<PathBuf> {
        if let Some(custom) = &self.data_dir {
            Ok(expand_path(custom))
        } else {
            let base = dirs::data_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
            let dotdog_d = base.join("dotdog");
            let dotmatrix_d = base.join("dotmatrix");
            if dotdog_d.exists() {
                Ok(dotdog_d)
            } else if dotmatrix_d.exists() {
                Ok(dotmatrix_d)
            } else {
                Ok(dotdog_d)
            }
        }
    }

    /// Get the store directory path (git-tracked file store)
    pub fn store_dir(&self) -> anyhow::Result<PathBuf> {
        Ok(self.data_dir()?.join("store"))
    }

    /// Get the backups directory path
    pub fn backups_dir(&self) -> anyhow::Result<PathBuf> {
        Ok(self.data_dir()?.join("backups"))
    }

    /// Get the directory for a specific project
    pub fn project_dir(&self, project_name: &str) -> anyhow::Result<PathBuf> {
        let data_dir = self.data_dir()?;
        Ok(data_dir.join("projects").join(project_name))
    }

    /// Get the store directory for a specific project
    pub fn project_store_dir(&self, project_name: &str) -> anyhow::Result<PathBuf> {
        Ok(self.project_dir(project_name)?.join("store"))
    }

    /// Get the index path for a specific project
    pub fn project_index_path(&self, project_name: &str) -> anyhow::Result<PathBuf> {
        Ok(self.project_dir(project_name)?.join("index.json"))
    }
}

/// Expand ~ to home directory
pub fn expand_path(path: &str) -> PathBuf {
    if path.starts_with("~/") || path == "~" {
        if let Some(home) = dirs::home_dir() {
            if path == "~" {
                return home;
            }
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

/// Contract home directory to ~
pub fn contract_path(path: &std::path::Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}
