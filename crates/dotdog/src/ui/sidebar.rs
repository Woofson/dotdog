//! NoteDog-style 2-Level Navigation Sidebar for dotmatrix TUI
//!
//! Top Pane: Projects List (with health badges, track mode, encryption status)
//! Bottom Pane: Tracked Files List in the active project

use crate::app::{App, Pane};
use crate::theme::Theme;
use dmcore::{FileStatus, TrackMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem},
    Frame,
};

pub fn render_sidebar(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    // Split Sidebar Vertically using layout configuration
    let projects_pct = dmcore::parse_constraint(&app.config.layout.projects_height, 45);
    let files_pct = dmcore::parse_constraint(&app.config.layout.files_height, 55);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(projects_pct), Constraint::Percentage(files_pct)])
        .split(area);

    let projects_area = chunks[0];
    let files_area = chunks[1];

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Top Pane: Projects List
    // ─────────────────────────────────────────────────────────────────────────
    let is_projects_focused = app.focused_pane == Pane::Projects;

    let project_items: Vec<ListItem> = app
        .projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let is_selected = i == app.active_project_idx;

            let (status_glyph, status_color) = if project.summary.errors > 0 {
                ("!", Color::Red)
            } else if project.summary.missing > 0 {
                ("✗", Color::LightRed)
            } else if project.summary.drifted > 0 {
                ("⚠", Color::Yellow)
            } else if project.summary.new > 0 {
                ("+", Color::Cyan)
            } else {
                ("✓", Color::Green)
            };

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

            let primary_mode = if both_cnt > 0 || (git_cnt > 0 && backup_cnt > 0) {
                ("[+]", Color::Green)
            } else if git_cnt > 0 {
                ("[G]", Color::Cyan)
            } else if backup_cnt > 0 {
                ("[B]", Color::Magenta)
            } else {
                ("   ", Color::DarkGray)
            };

            let proj_icon = app.config.icons.get_project_icon(&project.name);
            let enc_tag = if enc_cnt > 0 { " 🔒" } else { "" };
            let prefix = if is_selected { "▶ " } else { "  " };

            let name_style = if is_selected {
                theme.item_style(true)
            } else {
                Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", status_glyph), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", proj_icon.trim()), Style::default().fg(theme.sidebar_title)),
                Span::styled(project.name.clone(), name_style),
                Span::styled(enc_tag, Style::default().fg(theme.encrypted_tag)),
                Span::styled(format!(" ({}) ", project.file_count), Style::default().fg(Color::DarkGray)),
                Span::styled(primary_mode.0, Style::default().fg(primary_mode.1).add_modifier(Modifier::BOLD)),
            ]);

            ListItem::new(line).style(theme.item_bg_style(is_selected))
        })
        .collect();

    let projects_title = format!(" 📦 PROJECTS ({}) ", app.projects.len());
    let projects_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if is_projects_focused {
            theme.active_sidebar_border_style()
        } else {
            theme.border_style()
        })
        .title(Span::styled(
            projects_title,
            if is_projects_focused { theme.sidebar_title_style() } else { theme.title_style() },
        ));

    let projects_list = List::new(project_items)
        .block(projects_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(projects_list, projects_area, &mut app.project_list_state);

    // ─────────────────────────────────────────────────────────────────────────
    // 2. Bottom Pane: Tracked Files List in Active Project
    // ─────────────────────────────────────────────────────────────────────────
    let is_files_focused = app.focused_pane == Pane::Files;

    let active_project = app.projects.get(app.active_project_idx);
    let files_title = if let Some(proj) = active_project {
        format!(" 📋 FILES: {} ({}) ", proj.name, proj.files.len())
    } else {
        " 📋 TRACKED FILES (0) ".to_string()
    };

    let file_items: Vec<ListItem> = if let Some(proj) = active_project {
        proj.files
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let is_selected = i == app.active_file_idx;

                let (status_glyph, status_color) = match file.status {
                    FileStatus::Synced => ("✓", Color::Green),
                    FileStatus::Drifted => ("⚠", Color::Yellow),
                    FileStatus::New => ("+", Color::Cyan),
                    FileStatus::Missing => ("✗", Color::LightRed),
                    FileStatus::Error => ("!", Color::Red),
                };

                let (mode_badge, mode_color) = match file.track_mode {
                    TrackMode::Git => ("[G]", Color::Cyan),
                    TrackMode::Backup => ("[B]", Color::Magenta),
                    TrackMode::Both => ("[+]", Color::Green),
                };

                let enc_glyph = if file.encrypted { " 🔒" } else { "" };
                let size_str = file.size.map(crate::app::format_size).unwrap_or_default();
                let prefix = if is_selected { "▶ " } else { "  " };

                let filename = if let Some(idx) = file.path.rfind('/') {
                    &file.path[idx + 1..]
                } else {
                    &file.path
                };

                let line = Line::from(vec![
                    Span::styled(prefix, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", status_glyph), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                    Span::styled(format!("{} ", mode_badge), Style::default().fg(mode_color)),
                    Span::styled(
                        filename,
                        if is_selected { theme.item_style(true) } else { theme.fg_style() },
                    ),
                    Span::styled(enc_glyph, Style::default().fg(theme.encrypted_tag)),
                    Span::styled(format!("  {}", size_str), Style::default().fg(Color::DarkGray)),
                ]);

                ListItem::new(line).style(theme.item_bg_style(is_selected))
            })
            .collect()
    } else {
        Vec::new()
    };

    let files_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if is_files_focused {
            theme.active_sidebar_border_style()
        } else {
            theme.border_style()
        })
        .title(Span::styled(
            files_title,
            if is_files_focused { theme.sidebar_title_style() } else { theme.title_style() },
        ));

    let files_list = List::new(file_items)
        .block(files_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(files_list, files_area, &mut app.file_list_state);
}
