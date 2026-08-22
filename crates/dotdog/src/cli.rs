//! CLI subcommands module for dmx
//!
//! Handles all headless CLI commands (status, backup, add, sync, list, etc.)

use age::secrecy::SecretString;
use clap::{Parser, Subcommand, ValueEnum};
use dmcore::{
    backup_archive, backup_project_incremental_encrypted_with_message, contract_path, expand_path,
    get_remote_url, hash_file, init_project_repo, list_archives, pull, push, recent_commits,
    retrieve_file_from_encrypted, scan_project, set_remote_url, ArchiveFormat, Config, FileStatus,
    Index, Manifest, Project, ProjectSummary, TrackMode, TrackedFile,
};
use std::io::BufRead;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(name = "dotdog", bin_name = "dotdog")]
#[command(author = "Bolt J Woofson <https://github.com/Woofson/dotdog>")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "DotDog 🐶 - Modern dotfile compositor with git versioning & DotDog TUI (formerly Dot Matrix)")]
#[command(after_help = "Run 'dotdog' without arguments to launch the interactive DotDog TUI.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output as JSON (for scripting and automation)
    #[arg(long, global = true)]
    pub json: bool,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
pub enum Commands {
    /// Launch the interactive DotDog TUI (default when run without arguments)
    Tui,

    /// Initialize dotmatrix configuration and directories
    Init,

    /// Create a new dotfile / project repository
    New {
        /// Project name
        name: String,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// Add files to a project
    Add {
        /// Project name
        project: String,

        /// File paths to track
        files: Vec<String>,

        /// Track mode for added files (git, backup, both)
        #[arg(short, long, value_enum, default_value = "git")]
        track: TrackModeArg,

        /// Mark added files as encrypted with Age
        #[arg(short, long)]
        encrypted: bool,
    },

    /// Remove / untrack files from a project
    Remove {
        /// Project name
        project: String,

        /// File paths to untrack
        files: Vec<String>,
    },

    /// Show project status and file drift
    Status {
        /// Project name (or all projects if omitted)
        project: Option<String>,

        /// Show only files needing attention (drifted, new, missing)
        #[arg(short, long)]
        changes: bool,
    },

    /// Sync drifted files to index (update hashes)
    Sync {
        /// Project name (or all projects if omitted)
        project: Option<String>,
    },

    /// Backup project files to content-addressed store / git
    Backup {
        /// Project name (or all projects if omitted)
        project: Option<String>,

        /// Custom commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Create archive backup (.tar.gz, .zip, .7z) instead of git commit
        #[arg(short, long)]
        archive: bool,

        /// Archive format (tar.gz, zip, 7z)
        #[arg(short, long, value_enum)]
        format: Option<ArchiveFormatArg>,

        /// Read encryption password from file
        #[arg(long)]
        password_file: Option<PathBuf>,

        /// Read encryption password from stdin
        #[arg(long)]
        password_stdin: bool,
    },

    /// Restore files from a backup commit or archive
    Restore {
        /// Project name
        project: String,

        /// Specific file paths to restore (restores all if omitted)
        files: Vec<String>,

        /// Output directory (defaults to original location)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Read encryption password from file
        #[arg(long)]
        password_file: Option<PathBuf>,

        /// Read encryption password from stdin
        #[arg(long)]
        password_stdin: bool,
    },

    /// List projects, tracked files, or backup archives
    List {
        /// Item type to list (projects, files, archives, commits)
        #[arg(value_enum, default_value = "projects")]
        item: ListItemArg,

        /// Project name (for files, archives, commits)
        project: Option<String>,
    },

    /// Clean up missing/deleted files from manifest
    Clean {
        /// Project name
        project: String,
    },

    /// Acknowledge missing files (keep tracked, mute drift warnings)
    Ack {
        /// Project name
        project: String,

        /// Specific file path (or all missing if omitted)
        file: Option<String>,
    },

    /// Configure Git remote repository URL
    Remote {
        /// Project name
        project: String,

        /// Git remote URL (or omit to show current remote)
        url: Option<String>,
    },

    /// Push project git commits to remote repository
    Push {
        /// Project name
        project: String,
    },

    /// Pull project git commits from remote repository
    Pull {
        /// Project name
        project: String,
    },

    /// Verify file integrity and hashes across all projects
    Verify {
        /// Project name (or all projects if omitted)
        project: Option<String>,
    },
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackModeArg {
    Git,
    Backup,
    Both,
}

impl From<TrackModeArg> for TrackMode {
    fn from(arg: TrackModeArg) -> Self {
        match arg {
            TrackModeArg::Git => TrackMode::Git,
            TrackModeArg::Backup => TrackMode::Backup,
            TrackModeArg::Both => TrackMode::Both,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArchiveFormatArg {
    #[value(name = "tar.gz", alias = "targz")]
    TarGz,
    Zip,
    #[value(name = "7z", alias = "sevenz")]
    SevenZ,
}

impl From<ArchiveFormatArg> for ArchiveFormat {
    fn from(arg: ArchiveFormatArg) -> Self {
        match arg {
            ArchiveFormatArg::TarGz => ArchiveFormat::TarGz,
            ArchiveFormatArg::Zip => ArchiveFormat::Zip,
            ArchiveFormatArg::SevenZ => ArchiveFormat::SevenZ,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListItemArg {
    Projects,
    Files,
    Archives,
    Commits,
}

pub fn run_cli(command: Commands, json: bool) -> anyhow::Result<()> {
    match command {
        Commands::Tui => Ok(()),
        Commands::Init => cmd_init(json),
        Commands::New { name, description } => cmd_new(&name, description.as_deref(), json),
        Commands::Add {
            project,
            files,
            track,
            encrypted,
        } => cmd_add(&project, &files, track.into(), encrypted, json),
        Commands::Remove { project, files } => cmd_remove(&project, &files, json),
        Commands::Status { project, changes } => cmd_status(project.as_deref(), changes, json),
        Commands::Sync { project } => cmd_sync(project.as_deref(), json),
        Commands::Backup {
            project,
            message,
            archive,
            format,
            password_file,
            password_stdin,
        } => cmd_backup(
            project.as_deref(),
            message.as_deref(),
            archive,
            format.map(Into::into),
            password_file.as_deref(),
            password_stdin,
            json,
        ),
        Commands::Restore {
            project,
            files,
            output,
            password_file,
            password_stdin,
        } => cmd_restore(
            &project,
            &files,
            output.as_deref(),
            password_file.as_deref(),
            password_stdin,
            json,
        ),
        Commands::List { item, project } => cmd_list(item, project.as_deref(), json),
        Commands::Clean { project } => cmd_clean(&project, json),
        Commands::Ack { project, file } => cmd_ack(&project, file.as_deref(), json),
        Commands::Remote { project, url } => cmd_remote(&project, url.as_deref(), json),
        Commands::Push { project } => cmd_push(&project, json),
        Commands::Pull { project } => cmd_pull(&project, json),
        Commands::Verify { project } => cmd_verify(project.as_deref(), json),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Command implementations
// ─────────────────────────────────────────────────────────────────────────────

fn cmd_init(json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let manifest = Manifest::load()?;
    let index = Index::load()?;

    if json {
        let output = serde_json::json!({
            "status": "ok",
            "config_dir": Config::config_dir()?.to_string_lossy(),
            "data_dir": config.data_dir()?.to_string_lossy(),
            "projects_count": manifest.projects.len(),
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("✓ DotDog initialized successfully.");
        println!("  Config path: {:?}", Config::config_path()?);
        println!("  Themes path: {:?}", Config::themes_dir()?);
        println!("  Data path:   {:?}", config.data_dir()?);
    }
    let _ = index;
    Ok(())
}

fn cmd_new(name: &str, description: Option<&str>, json: bool) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    if manifest.projects.contains_key(name) {
        anyhow::bail!("Project '{}' already exists.", name);
    }

    let mut project = Project::new();
    if let Some(desc) = description {
        project.description = Some(desc.to_string());
    }

    let config = Config::load()?;
    if config.git_enabled {
        let _ = init_project_repo(&config, name);
    }

    manifest.add_project(name.to_string(), project);
    manifest.save()?;

    if json {
        let out = serde_json::json!({ "status": "created", "project": name });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Created project '{}'", name);
    }
    Ok(())
}

fn cmd_add(
    project_name: &str,
    files: &[String],
    track_mode: TrackMode,
    encrypted: bool,
    json: bool,
) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let project = manifest
        .get_project_mut(project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project_name))?;

    let mut added = Vec::new();
    for file_str in files {
        let p = expand_path(file_str);
        if !p.exists() {
            eprintln!("Warning: File '{}' does not exist on disk.", file_str);
        }
        let contracted = contract_path(&p);
        let mut tf = TrackedFile::new(&contracted);
        tf.track = track_mode;
        tf.encrypted = encrypted;
        project.add_file(tf);
        added.push(contracted);
    }

    manifest.save()?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "project": project_name,
            "added_count": added.len(),
            "files": added,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Added {} file(s) to '{}'", added.len(), project_name);
        for f in &added {
            let enc_tag = if encrypted { " [🔒 Encrypted]" } else { "" };
            println!("  + {}{}", f, enc_tag);
        }
    }
    Ok(())
}

fn cmd_remove(project_name: &str, files: &[String], json: bool) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let project = manifest
        .get_project_mut(project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project_name))?;

    let mut removed = Vec::new();
    for file_str in files {
        let p = expand_path(file_str);
        let contracted = contract_path(&p);
        project.remove_file(&contracted);
        removed.push(contracted);
    }

    manifest.save()?;

    if json {
        let out = serde_json::json!({
            "status": "ok",
            "project": project_name,
            "removed": removed,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Removed {} file(s) from '{}'", removed.len(), project_name);
        for f in &removed {
            println!("  - {}", f);
        }
    }
    Ok(())
}

fn cmd_status(project_filter: Option<&str>, changes_only: bool, json: bool) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;

    let mut results = Vec::new();

    for (name, project) in &manifest.projects {
        if let Some(pf) = project_filter {
            if name != pf {
                continue;
            }
        }

        let index = Index::load_for_project(&config, name).unwrap_or_default();
        let scanned = scan_project(project, &index);
        let summary = ProjectSummary::from_results(&scanned);
        results.push((name.clone(), project.clone(), scanned, summary));
    }

    if json {
        let out = serde_json::json!({
            "projects": results.iter().map(|(name, p, scanned, summary)| {
                serde_json::json!({
                    "name": name,
                    "description": p.description,
                    "synced": summary.synced,
                    "drifted": summary.drifted,
                    "new": summary.new,
                    "missing": summary.missing,
                    "errors": summary.errors,
                    "files": scanned.iter().map(|f| {
                        serde_json::json!({
                            "path": f.path,
                            "status": format!("{:?}", f.status),
                            "track_mode": format!("{:?}", f.track_mode),
                            "size": f.current_size,
                        })
                    }).collect::<Vec<_>>()
                })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
        return Ok(());
    }

    for (name, p, scanned, summary) in &results {
        println!("📦 Project: {}", name);
        if let Some(ref desc) = p.description {
            println!("   Description: {}", desc);
        }
        println!(
            "   Summary: {} synced, {} drifted, {} new, {} missing",
            summary.synced, summary.drifted, summary.new, summary.missing
        );
        for f in scanned {
            if changes_only && f.status == FileStatus::Synced {
                continue;
            }
            let (glyph, tag) = match f.status {
                FileStatus::Synced => ("✓", "Synced"),
                FileStatus::Drifted => ("⚠", "Drifted"),
                FileStatus::New => ("+", "New"),
                FileStatus::Missing => ("✗", "Missing"),
                FileStatus::Error => ("!", "Error"),
            };
            println!("   {} [{}] {}", glyph, tag, f.path);
        }
        println!();
    }

    Ok(())
}

fn cmd_sync(project_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;

    let mut synced_count = 0;
    for (name, project) in &manifest.projects {
        if let Some(pf) = project_filter {
            if name != pf {
                continue;
            }
        }
        let mut index = Index::load_for_project(&config, name).unwrap_or_default();
        for file in &project.files {
            let abs_path = expand_path(&file.path);
            if abs_path.exists() {
                let hash = hash_file(&abs_path).unwrap_or_default();
                let meta = dmcore::file_metadata(&abs_path).ok();
                let (size, modified) = meta.unwrap_or((0, 0));
                index.upsert(abs_path, dmcore::FileEntry {
                    hash,
                    size,
                    modified,
                    last_sync: Some(chrono::Utc::now()),
                    last_backup: None,
                    encrypted: file.encrypted,
                });
                synced_count += 1;
            }
        }
        index.save_for_project(&config, name)?;
    }

    if json {
        let out = serde_json::json!({
            "status": "synced",
            "count": synced_count,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Synced {} file hashes into index.", synced_count);
    }
    Ok(())
}

fn cmd_backup(
    project_filter: Option<&str>,
    message: Option<&str>,
    archive: bool,
    format: Option<ArchiveFormat>,
    password_file: Option<&Path>,
    password_stdin: bool,
    json: bool,
) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;

    let password = resolve_password(password_file, password_stdin);

    for (name, project) in &manifest.projects {
        if let Some(pf) = project_filter {
            if name != pf {
                continue;
            }
        }

        if archive {
            let fmt = format.unwrap_or(config.default_archive_format);
            let path = backup_archive(&config, name, project, fmt)?;
            if json {
                let out = serde_json::json!({
                    "status": "archive_created",
                    "project": name,
                    "archive_path": path.to_string_lossy(),
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("✓ Archive created for '{}': {:?}", name, path);
            }
        } else {
            let res = backup_project_incremental_encrypted_with_message(
                &config,
                name,
                project,
                password.as_ref(),
                message,
            )?;

            if json {
                let out = serde_json::json!({
                    "status": "committed",
                    "project": name,
                    "files_backed_up": res.backed_up,
                    "bytes_written": res.bytes_stored,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!(
                    "✓ Backup committed for '{}' ({} files backed up, {} bytes)",
                    name, res.backed_up, res.bytes_stored
                );
            }
        }
    }

    Ok(())
}

fn cmd_restore(
    project_name: &str,
    files: &[String],
    output: Option<&Path>,
    password_file: Option<&Path>,
    password_stdin: bool,
    json: bool,
) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;
    let project = manifest
        .get_project(project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project_name))?;

    let store_dir = config.project_store_dir(project_name)?;
    let index = Index::load_for_project(&config, project_name)?;
    let password = resolve_password(password_file, password_stdin);

    let mut restored = Vec::new();

    let files_to_restore: Vec<String> = if files.is_empty() {
        project.files.iter().map(|f| f.path.clone()).collect()
    } else {
        files.to_vec()
    };

    for file_rel in &files_to_restore {
        let dest_path = if let Some(out) = output {
            out.join(Path::new(file_rel).file_name().unwrap_or_default())
        } else {
            expand_path(file_rel)
        };

        let tf = project.files.iter().find(|f| f.path == *file_rel);
        let is_enc = tf.map(|t| t.encrypted).unwrap_or(false);

        let abs_path = expand_path(file_rel);
        if let Some(entry) = index.get(&abs_path) {
            let ok = retrieve_file_from_encrypted(
                &store_dir,
                &entry.hash,
                &dest_path,
                password.as_ref(),
                is_enc,
            )?;
            if ok {
                restored.push((file_rel.clone(), dest_path));
            }
        }
    }

    if json {
        let out = serde_json::json!({
            "status": "restored",
            "project": project_name,
            "restored_files": restored.iter().map(|(r, d)| {
                serde_json::json!({ "rel": r, "dest": d.to_string_lossy() })
            }).collect::<Vec<_>>()
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Restored {} file(s) for project '{}':", restored.len(), project_name);
        for (r, d) in &restored {
            println!("  • {} -> {:?}", r, d);
        }
    }

    Ok(())
}

fn cmd_list(item: ListItemArg, project_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;

    match item {
        ListItemArg::Projects => {
            if json {
                let out = serde_json::json!({
                    "projects": manifest.projects.iter().map(|(name, p)| {
                        serde_json::json!({
                            "name": name,
                            "description": p.description,
                            "files_count": p.files.len(),
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("📦 Configured Projects ({}):", manifest.projects.len());
                for (name, p) in &manifest.projects {
                    let desc = p.description.as_deref().unwrap_or("");
                    println!("  • {:<20} ({} files) {}", name, p.files.len(), desc);
                }
            }
        }
        ListItemArg::Files => {
            let mut list = Vec::new();
            for (name, p) in &manifest.projects {
                if let Some(pf) = project_filter {
                    if name != pf {
                        continue;
                    }
                }
                for f in &p.files {
                    list.push((name.clone(), f.path.clone(), f.encrypted, f.track));
                }
            }

            if json {
                let out = serde_json::json!({
                    "files": list.iter().map(|(proj, path, enc, mode)| {
                        serde_json::json!({
                            "project": proj,
                            "path": path,
                            "encrypted": enc,
                            "track_mode": format!("{:?}", mode),
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("📄 Tracked Files ({}):", list.len());
                for (proj, path, enc, mode) in &list {
                    let enc_str = if *enc { " 🔒" } else { "" };
                    println!("  [{}] {:<30} [{:?}]{}", proj, path, mode, enc_str);
                }
            }
        }
        ListItemArg::Archives => {
            let proj_name = project_filter.ok_or_else(|| anyhow::anyhow!("Must specify project name for archives list"))?;
            let archives = list_archives(&config, proj_name)?;
            if json {
                let out = serde_json::json!({
                    "project": proj_name,
                    "archives": archives.iter().map(|a| a.path.to_string_lossy()).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("📦 Archives for '{}':", proj_name);
                for a in &archives {
                    println!("  • {:?}", a.path);
                }
            }
        }
        ListItemArg::Commits => {
            let proj_name = project_filter.ok_or_else(|| anyhow::anyhow!("Must specify project name for commits list"))?;
            let project_dir = config.project_dir(proj_name)?;
            let commits = recent_commits(&project_dir, 20)?;
            if json {
                let out = serde_json::json!({
                    "project": proj_name,
                    "commits": commits.iter().map(|c| {
                        serde_json::json!({
                            "hash": c.short_hash,
                            "date": c.date,
                            "message": c.message,
                        })
                    }).collect::<Vec<_>>()
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!("🔄 Recent Commits for '{}':", proj_name);
                for c in &commits {
                    println!("  • {} ({}) {}", c.short_hash, c.date, c.message);
                }
            }
        }
    }
    Ok(())
}

fn cmd_clean(project_name: &str, json: bool) -> anyhow::Result<()> {
    let mut manifest = Manifest::load()?;
    let project = manifest
        .get_project_mut(project_name)
        .ok_or_else(|| anyhow::anyhow!("Project '{}' not found", project_name))?;

    let initial_count = project.files.len();
    project.files.retain(|f| expand_path(&f.path).exists());
    let removed_count = initial_count - project.files.len();

    manifest.save()?;

    if json {
        let out = serde_json::json!({
            "status": "cleaned",
            "project": project_name,
            "cleaned_count": removed_count,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Cleaned {} missing file(s) from project '{}'", removed_count, project_name);
    }
    Ok(())
}

fn cmd_ack(project_name: &str, file: Option<&str>, json: bool) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    if let Some(p) = manifest.get_project(project_name) {
        if let Some(target) = file {
            println!("✓ Acknowledged missing file '{}' for project '{}'", target, project_name);
        } else {
            let missing_count = p.files.iter().filter(|f| !expand_path(&f.path).exists()).count();
            println!("✓ Acknowledged {} missing file(s) for project '{}'", missing_count, project_name);
        }
    }

    if json {
        let out = serde_json::json!({ "status": "acknowledged", "project": project_name });
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    Ok(())
}

fn cmd_remote(project_name: &str, url: Option<&str>, json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let project_dir = config.project_dir(project_name)?;

    if let Some(new_url) = url {
        set_remote_url(&project_dir, new_url)?;
        if json {
            let out = serde_json::json!({ "status": "updated", "remote": new_url });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            println!("✓ Remote URL for '{}' set to: {}", project_name, new_url);
        }
    } else {
        let current_url = get_remote_url(&project_dir)?.unwrap_or_else(|| "No remote configured".to_string());
        if json {
            let out = serde_json::json!({ "project": project_name, "remote": current_url });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            println!("Git remote for '{}': {}", project_name, current_url);
        }
    }
    Ok(())
}

fn cmd_push(project_name: &str, json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let project_dir = config.project_dir(project_name)?;
    push(&project_dir)?;

    if json {
        let out = serde_json::json!({ "status": "pushed", "project": project_name });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Pushed project '{}' to Git remote.", project_name);
    }
    Ok(())
}

fn cmd_pull(project_name: &str, json: bool) -> anyhow::Result<()> {
    let config = Config::load()?;
    let project_dir = config.project_dir(project_name)?;
    pull(&project_dir)?;

    if json {
        let out = serde_json::json!({ "status": "pulled", "project": project_name });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("✓ Pulled changes for project '{}' from Git remote.", project_name);
    }
    Ok(())
}

fn cmd_verify(project_filter: Option<&str>, json: bool) -> anyhow::Result<()> {
    let manifest = Manifest::load()?;
    let config = Config::load()?;

    let mut total_files = 0;
    let mut verified = 0;
    let mut drifted = 0;
    let mut missing = 0;

    for (name, project) in &manifest.projects {
        if let Some(pf) = project_filter {
            if name != pf {
                continue;
            }
        }
        let index = Index::load_for_project(&config, name).unwrap_or_default();
        for file in &project.files {
            total_files += 1;
            let abs_path = expand_path(&file.path);
            if !abs_path.exists() {
                missing += 1;
            } else if let Some(indexed) = index.get(&abs_path) {
                let hash = hash_file(&abs_path).unwrap_or_default();
                if hash == indexed.hash {
                    verified += 1;
                } else {
                    drifted += 1;
                }
            } else {
                drifted += 1;
            }
        }
    }

    if json {
        let out = serde_json::json!({
            "total_files": total_files,
            "verified": verified,
            "drifted": drifted,
            "missing": missing,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("🔍 Verification Summary:");
        println!("   Total:    {} files", total_files);
        println!("   Verified: {} ✓", verified);
        println!("   Drifted:  {} ⚠", drifted);
        println!("   Missing:  {} ✗", missing);
    }
    Ok(())
}

fn resolve_password(password_file: Option<&Path>, password_stdin: bool) -> Option<SecretString> {
    if password_stdin {
        let mut line = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();
        if handle.read_line(&mut line).is_ok() {
            let trimmed = line.trim_end_matches(&['\r', '\n'][..]);
            if !trimmed.is_empty() {
                return Some(SecretString::from(trimmed.to_string()));
            }
        }
    }

    if let Some(path) = password_file {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim_end_matches(&['\r', '\n'][..]);
            if !trimmed.is_empty() {
                return Some(SecretString::from(trimmed.to_string()));
            }
        }
    }

    if let Ok(env_pass) = std::env::var("DOTDOG_PASSWORD").or_else(|_| std::env::var("DOTMATRIX_PASSWORD")) {
        if !env_pass.is_empty() {
            return Some(SecretString::from(env_pass));
        }
    }

    None
}
