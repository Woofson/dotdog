//! 2-Pane Restore & Version Diff View for dotmatrix TUI
//!
//! Inspired by NoteDog revision browser and diff modal.

use crate::app::{App, RestoreView};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_restore_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
    f.render_widget(Clear, area);

    if app.backup_projects.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(Span::styled("No backup repositories found on disk.", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("To create a backup, switch to ", theme.fg_style()),
                Span::styled("[Tab] Projects", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" and press ", theme.fg_style()),
                Span::styled("[a]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(" to create an incremental backup commit.", theme.fg_style()),
            ]),
        ])
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(theme.border_style())
                .title(Span::styled(" 🔄 RESTORE & VERSION DIFF ", theme.title_style())),
        );
        f.render_widget(msg, area);
        return;
    }

    // 2-Pane Split: Left History Navigation (42%) vs Right Commit Files & Diff (58%)
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(area);

    let left_area = chunks[0];
    let right_area = chunks[1];

    f.render_widget(Clear, left_area);
    f.render_widget(Clear, right_area);

    // Left pane depends on restore_view level
    match app.restore_view {
        RestoreView::Projects => render_backup_projects_list(f, left_area, right_area, app, theme),
        RestoreView::Commits | RestoreView::Files => render_commits_and_files(f, left_area, right_area, app, theme),
    }
}

fn render_backup_projects_list(f: &mut Frame, left_area: Rect, right_area: Rect, app: &mut App, theme: &Theme) {
    let items: Vec<ListItem> = app
        .backup_projects
        .iter()
        .enumerate()
        .map(|(i, project)| {
            let is_selected = app.backup_project_list_state.selected() == Some(i);
            let commits_str = format!("git:{}", project.commit_count);
            let archive_str = format!("arc:{}", project.archive_count);
            let last_backup_str = project
                .last_backup
                .as_ref()
                .map(|d| format!(" {}", d))
                .unwrap_or_default();

            let line = Line::from(vec![
                Span::styled("📦 ", Style::default().fg(theme.sidebar_title)),
                Span::styled(
                    format!("{:<18}", project.name),
                    if is_selected { theme.item_style(true) } else { Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD) },
                ),
                Span::styled(format!("{:>8} ", commits_str), Style::default().fg(Color::Cyan)),
                Span::styled(format!("{} ", archive_str), Style::default().fg(Color::Magenta)),
                Span::styled(last_backup_str, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(line).style(theme.item_bg_style(is_selected))
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.active_sidebar_border_style())
        .title(Span::styled(" 🔄 BACKUP PROJECTS ", theme.sidebar_title_style()));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(list, left_area, &mut app.backup_project_list_state);

    // Right Pane Overview
    let selected_project = app
        .backup_project_list_state
        .selected()
        .and_then(|i| app.backup_projects.get(i));

    let overview_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style())
        .title(Span::styled(" 📜 PROJECT SNAPSHOTS ", theme.title_style()));

    if let Some(proj) = selected_project {
        let text = vec![
            Line::from(vec![
                Span::styled(" Selected Backup: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(&proj.name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" Total Commits:   ", theme.fg_style()),
                Span::styled(format!("{} revisions", proj.commit_count), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(" Total Archives:  ", theme.fg_style()),
                Span::styled(format!("{} archives", proj.archive_count), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Press ", theme.fg_style()),
                Span::styled("[Enter] / [→] / [l]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(" to browse all past commits and restore individual files.", theme.fg_style()),
            ]),
        ];
        f.render_widget(Paragraph::new(text).block(overview_block), right_area);
    } else {
        f.render_widget(Paragraph::new("Select a backup project on the left.").block(overview_block), right_area);
    }
}

fn render_commits_and_files(f: &mut Frame, left_area: Rect, right_area: Rect, app: &mut App, theme: &Theme) {
    let project_name = app.selected_backup_project.clone().unwrap_or_default();

    // ─────────────────────────────────────────────────────────────────────────
    // Left Pane: Commits List
    // ─────────────────────────────────────────────────────────────────────────
    let commit_items: Vec<ListItem> = app
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let is_selected = app.commit_list_state.selected() == Some(i);
            let date_short = if commit.date.len() > 19 { &commit.date[..19] } else { &commit.date };

            let line = Line::from(vec![
                Span::styled(format!("{} ", commit.short_hash), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", date_short), Style::default().fg(Color::DarkGray)),
                Span::styled(commit.message.clone(), if is_selected { theme.item_style(true) } else { theme.fg_style() }),
            ]);

            ListItem::new(line).style(theme.item_bg_style(is_selected))
        })
        .collect();

    let commits_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.restore_view == RestoreView::Commits { theme.active_sidebar_border_style() } else { theme.border_style() })
        .title(Span::styled(format!(" 📜 {} BACKUPS (← Back) ", project_name), theme.sidebar_title_style()));

    let commit_list = List::new(commit_items)
        .block(commits_block)
        .highlight_style(theme.highlight_style());

    f.render_stateful_widget(commit_list, left_area, &mut app.commit_list_state);

    // ─────────────────────────────────────────────────────────────────────────
    // Right Pane: Files in Commit & Diff
    // ─────────────────────────────────────────────────────────────────────────
    if app.restore_files.is_empty() {
        let empty_p = Paragraph::new("Press [Enter] on a commit to load and inspect its files.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border_style())
                    .title(" 📋 COMMIT FILES & DIFF "),
            );
        f.render_widget(empty_p, right_area);
        return;
    }

    let file_items: Vec<ListItem> = app
        .restore_files
        .iter()
        .enumerate()
        .map(|(i, file)| {
            let is_selected = app.restore_list_state.selected() == Some(i);
            let is_checked = app.restore_selected.contains(&i);
            let check_glyph = if is_checked { "[*] " } else { "[ ] " };
            let check_color = if is_checked { Color::Green } else { Color::DarkGray };

            let (status_str, status_color) = if !file.exists_locally {
                ("NEW", Color::Cyan)
            } else if file.local_differs {
                ("CHG", Color::Yellow)
            } else {
                ("OK ", Color::Green)
            };

            let size_str = crate::app::format_size(file.size);
            let enc_tag = if file.encrypted { " 🔒" } else { "" };

            let line = Line::from(vec![
                Span::styled(check_glyph, Style::default().fg(check_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{} ", status_str), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{:>7} ", size_str), Style::default().fg(Color::DarkGray)),
                Span::styled(file.display_path.clone(), if is_selected { theme.item_style(true) } else { theme.fg_style() }),
                Span::styled(enc_tag, Style::default().fg(theme.encrypted_tag)),
            ]);

            ListItem::new(line).style(theme.item_bg_style(is_selected))
        })
        .collect();

    let commit_hash = app
        .selected_commit
        .and_then(|i| app.commits.get(i))
        .map(|c| c.short_hash.as_str())
        .unwrap_or("HEAD");

    let files_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(if app.restore_view == RestoreView::Files { theme.active_main_border_style() } else { theme.border_style() })
        .title(Span::styled(format!(" 📋 FILES AT COMMIT {} ({}) ", commit_hash, app.restore_files.len()), theme.main_title_style()));

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(2),
        ])
        .split(files_block.inner(right_area));

    f.render_widget(files_block, right_area);

    let files_list = List::new(file_items).highlight_style(theme.highlight_style());
    f.render_stateful_widget(files_list, right_chunks[0], &mut app.restore_list_state);

    let footer_hints = Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Restore  ", theme.fg_style()),
        Span::styled(" [Space] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Multi-select  ", theme.fg_style()),
        Span::styled(" [a/d] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("All/None  ", theme.fg_style()),
        Span::styled(" [v] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("View  ", theme.fg_style()),
        Span::styled(" [← / h] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Back", theme.fg_style()),
    ]);
    f.render_widget(Paragraph::new(footer_hints), right_chunks[1]);
}
