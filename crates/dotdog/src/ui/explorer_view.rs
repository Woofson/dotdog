//! File Explorer View embedded in the Main Workspace for adding files to the active project

use crate::app::{App, Pane};
use crate::theme::Theme;
use dmcore::TrackMode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

pub fn render_explorer_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    let is_focused = app.focused_pane == Pane::Main;
    let target_name = app.active_project_name().unwrap_or_else(|| "None".to_string());

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if is_focused {
            theme.active_main_border_style()
        } else {
            theme.border_style()
        })
        .title(Span::styled(
            format!(" 📂 ADD FILES TO: {} (Esc to return) ", target_name),
            if is_focused { theme.active_title_style() } else { theme.main_title_style() },
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Current Dir & Mode header
            Constraint::Min(6),    // Split: Left File list vs Right Selected file preview
            Constraint::Length(2), // Action keys footer
        ])
        .split(inner);

    // 1. Directory Path & Track Mode
    let path_display = if let Some(home) = dirs::home_dir() {
        if let Ok(rel) = app.browse_dir.strip_prefix(&home) {
            format!(" 📁 ~/{}", rel.display())
        } else {
            format!(" 📁 {}", app.browse_dir.display())
        }
    } else {
        format!(" 📁 {}", app.browse_dir.display())
    };

    let (mode_badge, mode_color) = match app.default_track_mode {
        TrackMode::Git => ("[G] Git", Color::Cyan),
        TrackMode::Backup => ("[B] Backup", Color::Magenta),
        TrackMode::Both => ("[+] Both", Color::Green),
    };

    let top_line = Line::from(vec![
        Span::styled(path_display, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("   │ Track Mode: ", theme.fg_style()),
        Span::styled(mode_badge, Style::default().fg(mode_color).add_modifier(Modifier::BOLD)),
        Span::styled(" [t] Cycle", Style::default().fg(Color::DarkGray)),
    ]);
    f.render_widget(Paragraph::new(top_line), chunks[0]);

    // 2. Middle Split: File List (55%) vs Live Preview (45%)
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);

    let items: Vec<ListItem> = app
        .browse_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = app.browse_list_state.selected() == Some(i);
            let size_str = file.size.map(crate::app::format_size).unwrap_or_default();

            if file.is_dir {
                if file.is_tracked() {
                    let proj_tag = format!(" [{}]", file.tracked_in.join(", "));
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
                let proj_tag = format!(" [{}]", file.tracked_in.join(", "));
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

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(" Files on Disk ");

    let list = List::new(items)
        .block(list_block)
        .highlight_style(theme.highlight_style());
    f.render_stateful_widget(list, mid_chunks[0], &mut app.browse_list_state);

    // Selected file preview on the right
    let selected_file = app
        .browse_list_state
        .selected()
        .and_then(|i| app.browse_files.get(i));

    if let Some(file) = selected_file {
        if file.is_dir {
            let dir_lines = vec![
                Line::from(vec![
                    Span::styled(" Directory: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(&file.name, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
                ]),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" • Press ", theme.fg_style()),
                    Span::styled("[Enter]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                    Span::styled(" to open folder", theme.fg_style()),
                ]),
                Line::from(vec![
                    Span::styled(" • Press ", theme.fg_style()),
                    Span::styled("[R]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                    Span::styled(" to recursively scan & add all files", theme.fg_style()),
                ]),
            ];
            let dir_block = Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme.border))
                .title(" Folder Actions ");
            f.render_widget(Paragraph::new(dir_lines).block(dir_block), mid_chunks[1]);
        } else {
            let preview_lines: Vec<Line> = if app.live_preview_content.is_empty() {
                vec![Line::from(Span::styled("  (Empty or unreadable file)", Style::default().fg(Color::DarkGray)))]
            } else {
                app.live_preview_content
                    .iter()
                    .take(mid_chunks[1].height as usize)
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
            f.render_widget(Paragraph::new(preview_lines).block(preview_block).wrap(Wrap { trim: false }), mid_chunks[1]);
        }
    } else {
        f.render_widget(Paragraph::new("No file selected.").block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded)), mid_chunks[1]);
    }

    // 3. Action keys footer
    let footer_line = Line::from(vec![
        Span::styled(" [Enter] / [a] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Add File / Enter Dir   ", theme.fg_style()),
        Span::styled(" [← / h / Bksp] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Parent Dir   ", theme.fg_style()),
        Span::styled(" [R] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Recursive Scan   ", theme.fg_style()),
        Span::styled(" [u] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Untrack   ", theme.fg_style()),
        Span::styled(" [Esc / q] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Done / Return", theme.fg_style()),
    ]);
    f.render_widget(Paragraph::new(footer_line), chunks[2]);
}
