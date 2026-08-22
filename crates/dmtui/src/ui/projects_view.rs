//! 2-Pane Projects View with Live Inspector & File Preview for dotmatrix TUI
//!
//! Inspired by NoteDog and Superfile multi-pane layout.

use crate::app::{App, ProjectViewItem};
use crate::theme::Theme;
use dmcore::{FileStatus, TrackMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_projects_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    if app.visible_items.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(Span::styled("No projects tracked yet.", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", theme.fg_style()),
                Span::styled("[n]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" to create your first project, or switch to ", theme.fg_style()),
                Span::styled("[Tab] Add Files", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                Span::styled(" to browse your filesystem.", theme.fg_style()),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style())
                .title(Span::styled(" 📦 PROJECTS ", theme.title_style())),
        );
        f.render_widget(msg, area);
        return;
    }

    // 2-Pane Split: Left Navigation (42%) vs Right Inspector & Preview (58%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    f.render_widget(Clear, left_area);
    f.render_widget(Clear, right_area);

    // ─────────────────────────────────────────────────────────────────────────
    // Left Pane: Projects & Files List
    // ─────────────────────────────────────────────────────────────────────────
    let items: Vec<ListItem> = app
        .visible_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = app.project_list_state.selected() == Some(i);

            match item {
                ProjectViewItem::Project {
                    name,
                    file_count,
                    summary,
                    expanded,
                } => {
                    let expand_glyph = if *expanded { "▼" } else { "▶" };
                    let (status_icon, status_color) = if summary.is_clean() {
                        ("✓", Color::Green)
                    } else {
                        ("⚠", Color::Yellow)
                    };

                    // Git remote status badge
                    let (git_badge, git_color) = if let Some(remote_status) = app.get_project_remote_status(name) {
                        if !remote_status.has_remote {
                            ("[no remote]".to_string(), Color::DarkGray)
                        } else if !remote_status.remote_reachable {
                            ("[offline]".to_string(), Color::LightRed)
                        } else if remote_status.ahead > 0 && remote_status.behind > 0 {
                            (format!("[↑{} ↓{}]", remote_status.ahead, remote_status.behind), Color::Yellow)
                        } else if remote_status.ahead > 0 {
                            (format!("[↑{}]", remote_status.ahead), Color::Cyan)
                        } else if remote_status.behind > 0 {
                            (format!("[↓{}]", remote_status.behind), Color::Magenta)
                        } else {
                            ("[synced]".to_string(), Color::Green)
                        }
                    } else {
                        (String::new(), Color::DarkGray)
                    };

                    let mut spans = vec![
                        Span::styled(format!("{} ", expand_glyph), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                        Span::styled("📦 ", Style::default().fg(theme.sidebar_title)),
                        Span::styled(status_icon, Style::default().fg(status_color)),
                        Span::raw(" "),
                        Span::styled(
                            name,
                            if is_selected {
                                theme.item_style(true)
                            } else {
                                Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)
                            },
                        ),
                        Span::styled(
                            format!(" ({})", file_count),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ];

                    if !git_badge.is_empty() {
                        spans.push(Span::raw(" "));
                        spans.push(Span::styled(git_badge, Style::default().fg(git_color)));
                    }

                    ListItem::new(Line::from(spans)).style(theme.item_bg_style(is_selected))
                }
                ProjectViewItem::File {
                    project_name,
                    path,
                    status,
                    size,
                    track_mode,
                    encrypted,
                    ..
                } => {
                    let is_ack = matches!(status, FileStatus::Missing)
                        && app.is_missing_acknowledged(project_name, path);

                    let (icon, color) = match status {
                        FileStatus::Synced => ("✓", Color::Green),
                        FileStatus::Drifted => ("⚠", Color::Yellow),
                        FileStatus::New => ("+", Color::Cyan),
                        FileStatus::Missing if is_ack => ("~", Color::DarkGray),
                        FileStatus::Missing => ("✗", Color::LightRed),
                        FileStatus::Error => ("!", Color::LightRed),
                    };

                    let size_str = size
                        .map(crate::app::format_size)
                        .unwrap_or_else(|| "-".to_string());

                    let (track_badge, track_color) = match track_mode {
                        TrackMode::Git => ("[G]", Color::Cyan),
                        TrackMode::Backup => ("[B]", Color::Magenta),
                        TrackMode::Both => ("[+]", Color::Green),
                    };

                    let enc_span = if *encrypted {
                        Span::styled(" [ENC]", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD))
                    } else {
                        Span::raw("")
                    };

                    let file_name = std::path::Path::new(path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());

                    let spans = vec![
                        Span::raw("    "),
                        Span::styled(format!("{} ", icon), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} ", track_badge), Style::default().fg(track_color)),
                        Span::styled(
                            file_name,
                            if is_selected {
                                theme.item_style(true)
                            } else {
                                theme.fg_style()
                            },
                        ),
                        enc_span,
                        Span::styled(format!("  {}", size_str), Style::default().fg(Color::DarkGray)),
                    ];

                    ListItem::new(Line::from(spans)).style(theme.item_bg_style(is_selected))
                }
            }
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_sidebar_border_style())
        .title(Span::styled(" 📦 PROJECTS & FILES ", theme.sidebar_title_style()));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(list, left_area, &mut app.project_list_state);

    // ─────────────────────────────────────────────────────────────────────────
    // Right Pane: Live Inspector & Preview
    // ─────────────────────────────────────────────────────────────────────────
    render_project_inspector(f, right_area, app, theme);
}

fn render_project_inspector(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let selected_item = app.selected_item();

    match selected_item {
        Some(ProjectViewItem::Project { name, file_count, summary, .. }) => {
            render_project_dashboard(f, area, app, name, *file_count, summary, theme);
        }
        Some(ProjectViewItem::File { project_name, path, abs_path, status, size, track_mode, encrypted }) => {
            render_file_live_preview(f, area, app, project_name, path, abs_path, *status, *size, *track_mode, *encrypted, theme);
        }
        None => {
            let empty_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style())
                .title(Span::styled(" 🔍 INSPECTOR ", theme.title_style()));
            let p = Paragraph::new("Select a project or file to inspect details and live preview.")
                .style(Style::default().fg(Color::DarkGray))
                .block(empty_block);
            f.render_widget(p, area);
        }
    }
}

fn render_project_dashboard(
    f: &mut Frame,
    area: Rect,
    app: &App,
    name: &str,
    file_count: usize,
    summary: &dmcore::ProjectSummary,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_main_border_style())
        .title(Span::styled(format!(" 📦 PROJECT: {} ", name), theme.main_title_style()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Status & Remote Info
            Constraint::Length(5), // Statistics breakdown cards
            Constraint::Min(6),    // Recent Git Commits log
            Constraint::Length(3), // Quick Actions hints
        ])
        .split(inner);

    // 1. Header & Git Remote Info
    let project_dir = app.config.project_dir(name).ok();
    let remote_url = project_dir.and_then(|d| dmcore::get_remote_url(&d).ok().flatten()).unwrap_or_default();
    let remote_display = if remote_url.is_empty() {
        "No remote repository set (press [G] to configure)".to_string()
    } else {
        remote_url
    };

    let remote_status_str = if let Some(rs) = app.get_project_remote_status(name) {
        if !rs.has_remote {
            "No remote configured"
        } else if !rs.remote_reachable {
            "Remote unreachable (offline)"
        } else if rs.ahead > 0 && rs.behind > 0 {
            "Ahead and behind remote"
        } else if rs.ahead > 0 {
            "Ahead of remote (ready to push [p])"
        } else if rs.behind > 0 {
            "Behind remote (ready to pull [P])"
        } else {
            "Up to date with remote ✓"
        }
    } else {
        "Unknown"
    };

    let header_lines = vec![
        Line::from(vec![
            Span::styled(" Git Remote: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(remote_display, Style::default().fg(theme.secondary)),
        ]),
        Line::from(vec![
            Span::styled(" Sync State: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(remote_status_str, Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // 2. Statistics Breakdown Cards
    let project_opt = app.manifest.get_project(name);
    let (git_count, backup_count, both_count, enc_count) = if let Some(p) = project_opt {
        let mut g = 0;
        let mut b = 0;
        let mut bt = 0;
        let mut enc = 0;
        for f in &p.files {
            match f.track {
                TrackMode::Git => g += 1,
                TrackMode::Backup => b += 1,
                TrackMode::Both => bt += 1,
            }
            if f.encrypted {
                enc += 1;
            }
        }
        (g, b, bt, enc)
    } else {
        (0, 0, 0, 0)
    };

    let stats_lines = vec![
        Line::from(vec![
            Span::styled(" Tracked Files: ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} total", file_count), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("  │  Clean: ", theme.fg_style()),
            Span::styled(format!("{}", summary.synced), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  │  Drifted: ", theme.fg_style()),
            Span::styled(format!("{}", summary.drifted), Style::default().fg(if summary.drifted > 0 { Color::Yellow } else { Color::DarkGray })),
            Span::styled("  │  Missing: ", theme.fg_style()),
            Span::styled(format!("{}", summary.missing), Style::default().fg(if summary.missing > 0 { Color::LightRed } else { Color::DarkGray })),
        ]),
        Line::from(vec![
            Span::styled(" Track Modes:   ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
            Span::styled(format!("[G] Git: {}  ", git_count), Style::default().fg(Color::Cyan)),
            Span::styled(format!("[B] Backup: {}  ", backup_count), Style::default().fg(Color::Magenta)),
            Span::styled(format!("[+] Both: {}  ", both_count), Style::default().fg(Color::Green)),
            Span::styled(format!("🔒 Encrypted: {}", enc_count), Style::default().fg(theme.encrypted_tag)),
        ]),
    ];
    let stats_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Statistics ");
    f.render_widget(Paragraph::new(stats_lines).block(stats_block), chunks[1]);

    // 3. Recent Commits Log
    let commit_items: Vec<ListItem> = if app.live_preview_project_commits.is_empty() {
        vec![ListItem::new(Span::styled("  No commits found in project repository. Press [a] to create a backup commit.", Style::default().fg(Color::DarkGray)))]
    } else {
        app.live_preview_project_commits
            .iter()
            .map(|c| {
                let date_short = if c.date.len() > 19 { &c.date[..19] } else { &c.date };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("  {} ", c.short_hash), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", date_short), Style::default().fg(Color::DarkGray)),
                    Span::styled(c.message.clone(), theme.fg_style()),
                ]))
            })
            .collect()
    };
    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Recent Backups & Commits ");
    f.render_widget(List::new(commit_items).block(commits_block), chunks[2]);

    // 4. Quick Actions
    let quick_actions = Line::from(vec![
        Span::styled(" [a] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Backup  ", theme.fg_style()),
        Span::styled(" [s] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Sync  ", theme.fg_style()),
        Span::styled(" [g] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Git Status  ", theme.fg_style()),
        Span::styled(" [p/P] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Push/Pull  ", theme.fg_style()),
        Span::styled(" [D] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Delete", theme.fg_style()),
    ]);
    f.render_widget(Paragraph::new(quick_actions), chunks[3]);
}

fn render_file_live_preview(
    f: &mut Frame,
    area: Rect,
    app: &App,
    project_name: &str,
    path: &str,
    abs_path: &std::path::Path,
    status: FileStatus,
    size: Option<u64>,
    track_mode: TrackMode,
    encrypted: bool,
    theme: &Theme,
) {
    let file_name = abs_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let (track_badge, track_color) = match track_mode {
        TrackMode::Git => ("[G] Git", Color::Cyan),
        TrackMode::Backup => ("[B] Backup", Color::Magenta),
        TrackMode::Both => ("[+] Both", Color::Green),
    };

    let (status_text, status_color) = match status {
        FileStatus::Synced => ("Synced ✓", Color::Green),
        FileStatus::Drifted => ("Drifted ⚠", Color::Yellow),
        FileStatus::New => ("New +", Color::Cyan),
        FileStatus::Missing => ("Missing on Disk ✗", Color::LightRed),
        FileStatus::Error => ("Error !", Color::LightRed),
    };

    let enc_badge = if encrypted { " 🔒 Encrypted" } else { " Plaintext" };
    let size_text = size.map(crate::app::format_size).unwrap_or_else(|| "-".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_main_border_style())
        .title(Span::styled(format!(" 📄 {} ({}) ", file_name, project_name), theme.main_title_style()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // File metadata header
            Constraint::Min(6),    // Live code preview
            Constraint::Length(1), // Footer keys
        ])
        .split(inner);

    // Meta Header
    let meta_lines = vec![
        Line::from(vec![
            Span::styled(" Path: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(path, Style::default().fg(theme.secondary)),
            Span::styled("  │  Size: ", theme.fg_style()),
            Span::styled(size_text, Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled(" Status: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(status_text, Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            Span::styled("  │  Mode: ", theme.fg_style()),
            Span::styled(track_badge, Style::default().fg(track_color).add_modifier(Modifier::BOLD)),
            Span::styled("  │ ", theme.fg_style()),
            Span::styled(enc_badge, Style::default().fg(if encrypted { theme.encrypted_tag } else { Color::DarkGray })),
        ]),
    ];
    f.render_widget(Paragraph::new(meta_lines), chunks[0]);

    // Live Code Preview
    let preview_lines: Vec<Line> = if app.live_preview_content.is_empty() {
        if encrypted {
            vec![Line::from(Span::styled("  🔒 This file is encrypted with age. Content is protected.", Style::default().fg(theme.encrypted_tag)))]
        } else if !abs_path.exists() {
            vec![Line::from(Span::styled("  ✗ File does not exist at local path on disk.", Style::default().fg(Color::LightRed)))]
        } else {
            vec![Line::from(Span::styled("  (Empty file)", Style::default().fg(Color::DarkGray)))]
        }
    } else {
        app.live_preview_content
            .iter()
            .take(chunks[1].height as usize)
            .map(|vl| {
                let spans: Vec<Span> = vl
                    .spans
                    .iter()
                    .map(|(text, style)| Span::styled(text.clone(), *style))
                    .collect();
                Line::from(spans)
            })
            .collect()
    };

    let preview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Live File Preview ");

    let preview_para = Paragraph::new(preview_lines)
        .block(preview_block)
        .wrap(Wrap { trim: false });
    f.render_widget(preview_para, chunks[1]);

    // Footer actions
    let file_footer = Line::from(vec![
        Span::styled(" [v] / [f] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Fullscreen Viewer   ", theme.fg_style()),
        Span::styled(" [m] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Mode   ", theme.fg_style()),
        Span::styled(" [x] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Encrypt   ", theme.fg_style()),
        Span::styled(" [s] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Sync", theme.fg_style()),
    ]);
    f.render_widget(Paragraph::new(file_footer), chunks[2]);
}
