//! 2-Pane Add Files & File Explorer View for dotmatrix TUI
//!
//! Inspired by NoteDog & Superfile multi-pane layout.

use crate::app::App;
use crate::theme::Theme;
use dmcore::TrackMode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_browse_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    // 2-Pane Split: Left File Explorer (52%) vs Right Target Project & Preview (48%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    f.render_widget(Clear, left_area);
    f.render_widget(Clear, right_area);

    // ─────────────────────────────────────────────────────────────────────────
    // Left Pane: File Explorer
    // ─────────────────────────────────────────────────────────────────────────
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1)])
        .split(left_area);

    let (mode_badge, mode_color) = match app.default_track_mode {
        TrackMode::Git => ("[G]", Color::Cyan),
        TrackMode::Backup => ("[B]", Color::Magenta),
        TrackMode::Both => ("[+]", Color::Green),
    };

    let items: Vec<ListItem> = app
        .browse_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = app.browse_list_state.selected() == Some(i);
            let size_str = file
                .size
                .map(crate::app::format_size)
                .unwrap_or_default();

            if file.is_dir {
                if file.is_tracked() {
                    let proj_tag = if file.tracked_in.len() == 1 {
                        format!(" [{}]", file.tracked_in[0])
                    } else {
                        format!(" [{} +{}]", file.tracked_in[0], file.tracked_in.len() - 1)
                    };
                    ListItem::new(Line::from(vec![
                        Span::styled(" ✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                        Span::styled("📁 ", Style::default().fg(theme.primary)),
                        Span::styled(
                            format!("{}/", file.name),
                            if is_selected { theme.item_style(true) } else { Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD) },
                        ),
                        Span::styled(proj_tag, Style::default().fg(theme.accent)),
                    ])).style(theme.item_bg_style(is_selected))
                } else {
                    ListItem::new(Line::from(vec![
                        Span::raw("   "),
                        Span::styled("📁 ", Style::default().fg(theme.primary)),
                        Span::styled(
                            format!("{}/", file.name),
                            if is_selected { theme.item_style(true) } else { Style::default().fg(theme.secondary) },
                        ),
                    ])).style(theme.item_bg_style(is_selected))
                }
            } else if file.is_tracked() {
                let proj_tag = if file.tracked_in.len() == 1 {
                    format!(" [{}]", file.tracked_in[0])
                } else {
                    format!(" [{}]", file.tracked_in.join(", "))
                };

                ListItem::new(Line::from(vec![
                    Span::styled(" ✓ ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                    Span::styled("📄 ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &file.name,
                        if is_selected { theme.item_style(true) } else { Style::default().fg(theme.primary) },
                    ),
                    Span::styled(format!("  {}", size_str), Style::default().fg(Color::DarkGray)),
                    Span::styled(proj_tag, Style::default().fg(theme.accent)),
                ])).style(theme.item_bg_style(is_selected))
            } else {
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{} ", mode_badge), Style::default().fg(mode_color)),
                    Span::styled("📄 ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        &file.name,
                        if is_selected { theme.item_style(true) } else { theme.fg_style() },
                    ),
                    Span::styled(format!("  {}", size_str), Style::default().fg(Color::DarkGray)),
                ])).style(theme.item_bg_style(is_selected))
            }
        })
        .collect();

    let target_name = app
        .target_project
        .as_ref()
        .or_else(|| app.projects.first().map(|p| &p.name))
        .map(|n| format!(" → 📦 {} ", n))
        .unwrap_or_default();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_sidebar_border_style())
        .title(Span::styled(format!(" 📂 FILE EXPLORER{} ", target_name), theme.sidebar_title_style()));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(list, left_chunks[0], &mut app.browse_list_state);

    // Current directory path breadcrumb
    let path_display = if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = app.browse_dir.strip_prefix(&home) {
            format!(" 📁 ~/{}", rel.display())
        } else {
            format!(" 📁 {}", app.browse_dir.display())
        }
    } else {
        format!(" 📁 {}", app.browse_dir.display())
    };
    let dir_line = Paragraph::new(path_display).style(Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD));
    f.render_widget(dir_line, left_chunks[1]);

    // ─────────────────────────────────────────────────────────────────────────
    // Right Pane: Target Project Card & File Preview
    // ─────────────────────────────────────────────────────────────────────────
    render_target_project_inspector(f, right_area, app, theme);
}

fn render_target_project_inspector(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let target_name = app
        .target_project
        .as_ref()
        .or_else(|| app.projects.first().map(|p| &p.name))
        .cloned()
        .unwrap_or_else(|| "None (press 'n' to create)".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_main_border_style())
        .title(Span::styled(format!(" 🎯 TARGET: {} ", target_name), theme.main_title_style()));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(4), // Target project & Mode settings
            Constraint::Min(6),    // Selected file preview
            Constraint::Length(4), // Action guide
        ])
        .split(inner);

    // 1. Target Card Header
    let (mode_badge, mode_desc, mode_color) = match app.default_track_mode {
        TrackMode::Git => ("[G] Git", "Version controlled & diffable", Color::Cyan),
        TrackMode::Backup => ("[B] Backup", "Incremental deduplicated backup", Color::Magenta),
        TrackMode::Both => ("[+] Both", "Git tracked and backed up", Color::Green),
    };

    let target_proj_info = app.manifest.get_project(&target_name);
    let target_file_count = target_proj_info.map(|p| p.file_count()).unwrap_or(0);

    let top_lines = vec![
        Line::from(vec![
            Span::styled(" Target Project: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(&target_name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({} tracked files)  ", target_file_count), Style::default().fg(Color::DarkGray)),
            Span::styled("[p] Cycle  [n] New", Style::default().fg(theme.secondary)),
        ]),
        Line::from(vec![
            Span::styled(" Add Track Mode: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(mode_badge, Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  ({})  ", mode_desc), Style::default().fg(Color::DarkGray)),
            Span::styled("[t] Cycle", Style::default().fg(theme.secondary)),
        ]),
    ];
    f.render_widget(Paragraph::new(top_lines), chunks[0]);

    // 2. Selected File Preview
    let selected_file = app
        .browse_list_state
        .selected()
        .and_then(|i| app.browse_files.get(i));

    if let Some(file) = selected_file {
        if file.is_dir {
            let dir_items = vec![
                Line::from(vec![
                    Span::styled(" Selected Folder: ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
                    Span::styled(&file.name, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" • Press ", theme.fg_style()),
                    Span::styled("[Enter]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(" to open and step inside this directory.", theme.fg_style()),
                ]),
                Line::from(vec![
                    Span::styled(" • Press ", theme.fg_style()),
                    Span::styled("[R]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(" to recursively scan all files in this folder and add them in batch.", theme.fg_style()),
                ]),
            ];
            let dir_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(" Folder Actions ");
            f.render_widget(Paragraph::new(dir_items).block(dir_block), chunks[1]);
        } else {
            let preview_lines: Vec<Line> = if app.live_preview_content.is_empty() {
                vec![Line::from(Span::styled("  (Empty or unreadable file)", Style::default().fg(Color::DarkGray)))]
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
                .title(format!(" Preview: {} ", file.name));

            let p = Paragraph::new(preview_lines)
                .block(preview_block)
                .wrap(Wrap { trim: false });
            f.render_widget(p, chunks[1]);
        }
    } else {
        let empty_p = Paragraph::new("No file selected.")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(theme.border_style()));
        f.render_widget(empty_p, chunks[1]);
    }

    // 3. Action Guide
    let guide_lines = vec![
        Line::from(vec![
            Span::styled(" [Enter] / [a] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Add File/Enter Dir   ", theme.fg_style()),
            Span::styled(" [R] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled("Recursive Scan   ", theme.fg_style()),
            Span::styled(" [u] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
            Span::styled("Untrack File", theme.fg_style()),
        ]),
        Line::from(vec![
            Span::styled(" [← / h / Bksp] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled("Parent Dir   ", theme.fg_style()),
            Span::styled(" [~] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("Home Dir   ", theme.fg_style()),
            Span::styled(" [v] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("View Full File", theme.fg_style()),
        ]),
    ];
    let guide_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));
    f.render_widget(Paragraph::new(guide_lines).block(guide_block), chunks[2]);
}
