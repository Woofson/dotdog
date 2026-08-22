//! Live Project Dashboard and Syntax-Highlighted File Inspector for dotmatrix TUI

use crate::app::{App, Pane};
use crate::theme::Theme;
use dmcore::{FileStatus, TrackMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_inspector_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    let is_main_focused = app.focused_pane == Pane::Main;
    let is_file_focused = app.focused_pane == Pane::Files;

    let active_project = app.projects.get(app.active_project_idx);

    if active_project.is_none() {
        let empty_p = Paragraph::new(vec![
            Line::from(Span::styled("No projects tracked yet.", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", theme.fg_style()),
                Span::styled("[n]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" to create your first project.", theme.fg_style()),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style())
                .title(Span::styled(" 🔍 PROJECT INSPECTOR ", theme.title_style())),
        );
        f.render_widget(empty_p, area);
        return;
    }

    let project = active_project.unwrap();
    let selected_file = project.files.get(app.active_file_idx);

    // If focused on files or if a file is explicitly selected, show the live file preview
    if is_file_focused && selected_file.is_some() {
        let file = selected_file.unwrap();
        render_file_preview_panel(f, area, app, &project.name, file, is_main_focused, theme);
    } else {
        render_project_dashboard_panel(f, area, app, project, is_main_focused, theme);
    }
}

fn render_project_dashboard_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    project: &crate::app::DisplayProject,
    is_focused: bool,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if is_focused {
            theme.active_main_border_style()
        } else {
            theme.border_style()
        })
        .title(Span::styled(
            format!(" 📦 PROJECT DASHBOARD: {} ", project.name),
            if is_focused { theme.active_title_style() } else { theme.main_title_style() },
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Remote info
            Constraint::Length(5), // Health breakdown
            Constraint::Min(6),    // Recent commits
            Constraint::Length(3), // Action bar
        ])
        .split(inner);

    // 1. Remote Info
    let project_dir = app.config.project_dir(&project.name).ok();
    let remote_url = project_dir
        .and_then(|d| dmcore::get_remote_url(&d).ok().flatten())
        .unwrap_or_default();

    let remote_display = if remote_url.is_empty() {
        "No remote configured (press [G] to set URL)".to_string()
    } else {
        remote_url
    };

    let remote_status_str = if let Some(rs) = app.get_project_remote_status(&project.name) {
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
            Span::styled(" Git Remote:  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(remote_display, Style::default().fg(theme.secondary)),
        ]),
        Line::from(vec![
            Span::styled(" Sync Health: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(remote_status_str, Style::default().fg(Color::Green)),
        ]),
    ];
    f.render_widget(Paragraph::new(header_lines), chunks[0]);

    // 2. Health & Track Modes Breakdown
    let (git_cnt, backup_cnt, both_cnt, enc_cnt) = {
        let mut g = 0;
        let mut b = 0;
        let mut bt = 0;
        let mut enc = 0;
        for f in &project.files {
            match f.track_mode {
                TrackMode::Git => g += 1,
                TrackMode::Backup => b += 1,
                TrackMode::Both => bt += 1,
            }
            if f.encrypted {
                enc += 1;
            }
        }
        (g, b, bt, enc)
    };

    let stats_lines = vec![
        Line::from(vec![
            Span::styled(" Tracked Files: ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} total", project.file_count), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("  │  Clean: ", theme.fg_style()),
            Span::styled(format!("{}", project.summary.synced), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled("  │  Drifted: ", theme.fg_style()),
            Span::styled(format!("{}", project.summary.drifted), Style::default().fg(if project.summary.drifted > 0 { Color::Yellow } else { Color::DarkGray })),
            Span::styled("  │  Missing: ", theme.fg_style()),
            Span::styled(format!("{}", project.summary.missing), Style::default().fg(if project.summary.missing > 0 { Color::LightRed } else { Color::DarkGray })),
        ]),
        Line::from(vec![
            Span::styled(" Track Modes:   ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
            Span::styled(format!("[G] Git: {}  ", git_cnt), Style::default().fg(Color::Cyan)),
            Span::styled(format!("[B] Backup: {}  ", backup_cnt), Style::default().fg(Color::Magenta)),
            Span::styled(format!("[+] Both: {}  ", both_cnt), Style::default().fg(Color::Green)),
            Span::styled(format!("🔒 Encrypted: {}", enc_cnt), Style::default().fg(theme.encrypted_tag)),
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
        vec![ListItem::new(Line::from(Span::styled("  (No commits yet — press [b] to create your first backup commit)", Style::default().fg(Color::DarkGray))))]
    } else {
        app.live_preview_project_commits
            .iter()
            .take(6)
            .map(|c| {
                let date_str = if c.date.len() > 19 { &c.date[..19] } else { &c.date };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("• {} ", c.short_hash), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("({}) ", date_str), Style::default().fg(Color::DarkGray)),
                    Span::styled(c.message.clone(), theme.fg_style()),
                ]))
            })
            .collect()
    };
    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Recent Revision History ");
    f.render_widget(List::new(commit_items).block(commits_block), chunks[2]);

    // 4. Action Guide
    let actions_line = Line::from(vec![
        Span::styled(" [+] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Add   ", theme.fg_style()),
        Span::styled(" [b] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Backup   ", theme.fg_style()),
        Span::styled(" [d] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Diffs   ", theme.fg_style()),
        Span::styled(" [s] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Sync   ", theme.fg_style()),
        Span::styled(" [e] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Encrypt   ", theme.fg_style()),
        Span::styled(" [p] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Push", theme.fg_style()),
    ]);
    let actions_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));
    f.render_widget(Paragraph::new(actions_line).block(actions_block), chunks[3]);
}

fn render_file_preview_panel(
    f: &mut Frame,
    area: Rect,
    app: &App,
    project_name: &str,
    file: &crate::app::DisplayFile,
    is_focused: bool,
    theme: &Theme,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if is_focused {
            theme.active_main_border_style()
        } else {
            theme.border_style()
        })
        .title(Span::styled(
            format!(" 📄 {} :: {} ", project_name, file.path),
            if is_focused { theme.active_title_style() } else { theme.main_title_style() },
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Metadata header
            Constraint::Min(4),    // Syntax-highlighted content
        ])
        .split(inner);

    // Metadata header
    let (status_str, status_color) = match file.status {
        FileStatus::Synced => ("Synced ✓", Color::Green),
        FileStatus::Drifted => ("Drifted ⚠", Color::Yellow),
        FileStatus::New => ("New +", Color::Cyan),
        FileStatus::Missing => ("Missing ✗", Color::LightRed),
        FileStatus::Error => ("Error !", Color::Red),
    };

    let size_str = file.size.map(crate::app::format_size).unwrap_or_else(|| "0 B".to_string());
    let enc_str = if file.encrypted { "🔒 Age Encrypted" } else { "Plaintext" };

    let meta_line = Line::from(vec![
        Span::styled(" Path: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}  ", file.abs_path.display()), theme.fg_style()),
        Span::styled("│ Size: ", Style::default().fg(theme.primary)),
        Span::styled(format!("{}  ", size_str), theme.fg_style()),
        Span::styled("│ Status: ", Style::default().fg(theme.primary)),
        Span::styled(format!("{}  ", status_str), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
        Span::styled("│ Security: ", Style::default().fg(theme.primary)),
        Span::styled(enc_str, Style::default().fg(theme.encrypted_tag)),
    ]);
    f.render_widget(Paragraph::new(meta_line), chunks[0]);

    // Syntax-highlighted file preview
    let preview_lines: Vec<Line> = if app.live_preview_content.is_empty() {
        vec![Line::from(Span::styled("  (Empty file, binary content, or missing on disk)", Style::default().fg(Color::DarkGray)))]
    } else {
        let max_lines = chunks[1].height as usize;
        app.live_preview_content
            .iter()
            .take(max_lines)
            .enumerate()
            .map(|(idx, vl)| {
                let gutter = format!("{:>3} │ ", idx + 1);
                let mut spans = vec![Span::styled(gutter, Style::default().fg(Color::DarkGray))];
                for (text, style) in &vl.spans {
                    spans.push(Span::styled(text.clone(), *style));
                }
                Line::from(spans)
            })
            .collect()
    };

    let preview_p = Paragraph::new(preview_lines).wrap(Wrap { trim: false });
    f.render_widget(preview_p, chunks[1]);
}
