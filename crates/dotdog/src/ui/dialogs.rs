//! Modal dialogs and popup overlays for dotmatrix TUI
//!
//! Styled to match NoteDog double/rounded modal aesthetics.

use crate::app::{PasswordPurpose, RecursivePreviewState, RestoreConfirmState, RestoreDestination, RestoreFile};
use crate::theme::Theme;
use dmcore::TrackMode;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

/// Helper to calculate centered rectangular area for modals
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Render passphrase input modal for age encryption
pub fn render_passphrase_modal(
    f: &mut Frame,
    area: Rect,
    purpose: PasswordPurpose,
    input_buffer: &str,
    theme: &Theme,
) {
    let popup_area = centered_rect(64, 28, area);
    f.render_widget(Clear, popup_area);

    let title = match purpose {
        PasswordPurpose::Backup => " 🔒 BACKUP ENCRYPTION PASSPHRASE ",
        PasswordPurpose::Restore => " 🔒 RESTORE DECRYPTION PASSPHRASE ",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            title,
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let label_str = match purpose {
        PasswordPurpose::Backup => "Enter passphrase to encrypt files with age:",
        PasswordPurpose::Restore => "Enter passphrase to decrypt protected backup files:",
    };
    let label = Paragraph::new(Span::styled(
        label_str,
        Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, inner_chunks[0]);

    let masked = "*".repeat(input_buffer.len());
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.secondary));

    let input_para = Paragraph::new(format!("{}█", masked)).block(input_block);
    f.render_widget(input_para, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Submit   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[3]);

    f.render_widget(block, popup_area);
}

/// Generic stylish single-line input dialog
pub fn render_input_dialog(
    f: &mut Frame,
    area: Rect,
    dialog_title: &str,
    input_label: &str,
    input_buffer: &str,
    placeholder: Option<&str>,
    theme: &Theme,
) {
    let popup_area = centered_rect(62, 25, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            format!(" 📝 {} ", dialog_title),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let label = Paragraph::new(Span::styled(
        input_label,
        Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
    ));
    f.render_widget(label, inner_chunks[0]);

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.secondary));

    let input_para = if input_buffer.is_empty() {
        if let Some(ph) = placeholder {
            Paragraph::new(Line::from(vec![
                Span::styled(ph, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ])).block(input_block)
        } else {
            Paragraph::new("█").block(input_block)
        }
    } else {
        Paragraph::new(format!("{}█", input_buffer)).block(input_block)
    };

    f.render_widget(input_para, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}

/// Delete confirmation modal
pub fn render_confirm_delete_modal(
    f: &mut Frame,
    area: Rect,
    project_name: &str,
    theme: &Theme,
) {
    let popup_area = centered_rect(60, 26, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.encrypted_tag))
        .title(Span::styled(
            " ⚠️ DELETE PROJECT ",
            Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(2),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let warning_text = format!("Are you sure you want to delete project '{}'?", project_name);
    let label = Paragraph::new(vec![
        Line::from(Span::styled(
            warning_text,
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "This will remove the project from manifest and delete its internal repository.",
            Style::default().fg(Color::LightRed),
        )),
    ]);
    f.render_widget(label, inner_chunks[0]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [y / Enter] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm Delete   ", theme.fg_style()),
        Span::styled(" [n / Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}

/// Restore confirmation dialog
pub fn render_restore_confirm_modal(
    f: &mut Frame,
    area: Rect,
    state: &RestoreConfirmState,
    restore_files: &[RestoreFile],
    theme: &Theme,
) {
    let popup_area = centered_rect(75, 75, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 🔄 CONFIRM RESTORE ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Destination options
            Constraint::Length(3), // Custom path input (if active)
            Constraint::Min(4),    // Files list
            Constraint::Length(2), // Summary & overwrite alert
            Constraint::Length(1), // Footer keys
        ])
        .split(popup_area);

    // Destination selector
    let orig_style = if state.destination == RestoreDestination::Original {
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        theme.fg_style()
    };
    let cust_style = if state.destination == RestoreDestination::Custom {
        Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)
    } else {
        theme.fg_style()
    };

    let dest_line = Line::from(vec![
        Span::styled(" Destination: ", Style::default().fg(theme.sidebar_title).add_modifier(Modifier::BOLD)),
        Span::styled(if state.destination == RestoreDestination::Original { " [*] [o] Original Path  " } else { " [ ] [o] Original Path  " }, orig_style),
        Span::styled(if state.destination == RestoreDestination::Custom { " [*] [c] Custom Path" } else { " [ ] [c] Custom Path" }, cust_style),
    ]);
    f.render_widget(Paragraph::new(dest_line), inner_chunks[0]);

    // Custom path box
    if state.destination == RestoreDestination::Custom {
        let path_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme.secondary))
            .title(" Custom Destination Directory ");
        let path_text = if state.custom_path.is_empty() {
            if state.entering_path { "█" } else { "~/restored_files (press [c] to edit)" }
        } else if state.entering_path {
            &format!("{}█", state.custom_path)
        } else {
            &state.custom_path
        };
        let path_para = Paragraph::new(path_text).block(path_block);
        f.render_widget(path_para, inner_chunks[1]);
    } else {
        let note = Paragraph::new(" Files will be restored to their original native file paths on disk.")
            .style(Style::default().fg(Color::DarkGray));
        f.render_widget(note, inner_chunks[1]);
    }

    // Files list
    let file_items: Vec<ListItem> = state
        .files_to_restore
        .iter()
        .filter_map(|&idx| restore_files.get(idx))
        .enumerate()
        .map(|(i, rf)| {
            let is_sel = i == state.selected_idx;
            let status_badge = if !rf.exists_locally {
                Span::styled("NEW ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else if rf.local_differs {
                Span::styled("CHG ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            } else {
                Span::styled("OK  ", Style::default().fg(Color::Green))
            };

            let line = Line::from(vec![
                Span::styled(if is_sel { " ▶ " } else { "   " }, Style::default().fg(theme.accent)),
                status_badge,
                Span::styled(rf.display_path.clone(), if is_sel { theme.highlight_style() } else { theme.fg_style() }),
            ]);

            ListItem::new(line).style(if is_sel { theme.item_bg_style(true) } else { Style::default() })
        })
        .collect();

    let files_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" Files to Restore ({}) ", state.files_to_restore.len()));
    let files_list = List::new(file_items).block(files_block);
    f.render_widget(files_list, inner_chunks[2]);

    // Overwrite alert
    let alert_spans = if state.will_overwrite > 0 {
        vec![
            Span::styled(format!(" ⚠️ {} existing file(s) will be OVERWRITTEN. ", state.will_overwrite), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
            Span::styled("  [b] View Backup  [l] View Local  [d] View Diff", Style::default().fg(theme.secondary)),
        ]
    } else {
        vec![
            Span::styled(" ✓ All files are new (no files will be overwritten).", Style::default().fg(Color::Green)),
        ]
    };
    f.render_widget(Paragraph::new(Line::from(alert_spans)), inner_chunks[3]);

    // Footer actions
    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [y / Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Confirm & Restore   ", theme.fg_style()),
        Span::styled(" [Tab] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Dest   ", theme.fg_style()),
        Span::styled(" [n / Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[4]);

    f.render_widget(block, popup_area);
}

/// Recursive scan & add files preview dialog
pub fn render_recursive_preview_modal(
    f: &mut Frame,
    area: Rect,
    preview: &mut RecursivePreviewState,
    target_project: Option<&str>,
    theme: &Theme,
) {
    let popup_area = centered_rect(75, 75, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            format!(" 📁 RECURSIVE ADD TO {} ", target_project.unwrap_or("Project")),
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let header = Line::from(vec![
        Span::styled(
            format!(" Found {} file(s) under: ", preview.preview_files.len()),
            Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD),
        ),
        Span::styled(preview.source_dir.display().to_string(), Style::default().fg(theme.secondary)),
    ]);
    f.render_widget(Paragraph::new(header), inner_chunks[0]);

    let items: Vec<ListItem> = preview
        .preview_files
        .iter()
        .enumerate()
        .map(|(i, pf)| {
            let is_checked = preview.selected_files.contains(&i);
            let check_str = if is_checked { "[*] " } else { "[ ] " };
            let check_color = if is_checked { Color::Green } else { Color::DarkGray };

            let track_str = match pf.track_mode {
                TrackMode::Git => "[G]",
                TrackMode::Backup => "[B]",
                TrackMode::Both => "[+]",
            };
            let track_color = match pf.track_mode {
                TrackMode::Git => Color::Cyan,
                TrackMode::Backup => Color::Magenta,
                TrackMode::Both => Color::Green,
            };

            let line = Line::from(vec![
                Span::styled(check_str, Style::default().fg(check_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", track_str), Style::default().fg(track_color)),
                Span::styled(pf.display_path.clone(), theme.fg_style()),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(format!(" Selected ({}/{}) ", preview.selected_files.len(), preview.preview_files.len()));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(list, inner_chunks[1], &mut preview.preview_list_state);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Add Selected   ", theme.fg_style()),
        Span::styled(" [Space] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle   ", theme.fg_style()),
        Span::styled(" [a] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("All   ", theme.fg_style()),
        Span::styled(" [t] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Cycle Mode   ", theme.fg_style()),
        Span::styled(" [Esc] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Cancel", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}
