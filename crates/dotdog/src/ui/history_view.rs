//! History, Diff, and Restore View embedded in the Main Workspace for the active project

use crate::app::{App, Pane, RestoreView};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};

pub fn render_history_view(f: &mut Frame, area: Rect, app: &mut App, theme: &Theme) {
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
            format!(" 🔄 REVISIONS & DIFFS: {} (Esc to return) ", target_name),
            if is_focused { theme.active_title_style() } else { theme.main_title_style() },
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Min(6),    // Split: Left Commits vs Right Files/Diff
            Constraint::Length(2), // Action keys footer
        ])
        .split(inner);

    // Split commits (40%) vs files (60%)
    let mid_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(chunks[0]);

    // 1. Commits list
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
        .title(Span::styled(format!(" 📜 Commits ({}) ", app.commits.len()), theme.sidebar_title_style()));

    let commit_list = List::new(commit_items)
        .block(commits_block)
        .highlight_style(theme.highlight_style());
    f.render_stateful_widget(commit_list, mid_chunks[0], &mut app.commit_list_state);

    // 2. Commit Files List / Diff
    if app.restore_files.is_empty() {
        let empty_p = Paragraph::new("Press [Enter] on a commit to load its files.")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(theme.border_style())
                    .title(" 📋 Commit Files "),
            );
        f.render_widget(empty_p, mid_chunks[1]);
    } else {
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

        let files_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(if app.restore_view == RestoreView::Files { theme.active_sidebar_border_style() } else { theme.border_style() })
            .title(Span::styled(format!(" 📋 Files ({}) ", app.restore_files.len()), theme.sidebar_title_style()));

        let files_list = List::new(file_items)
            .block(files_block)
            .highlight_style(theme.highlight_style());
        f.render_stateful_widget(files_list, mid_chunks[1], &mut app.restore_list_state);
    }

    // 3. Action keys footer
    let footer_line = Line::from(vec![
        Span::styled(" [Enter] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Select Commit / Restore   ", theme.fg_style()),
        Span::styled(" [Space] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Toggle Select   ", theme.fg_style()),
        Span::styled(" [a/d] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("All/None   ", theme.fg_style()),
        Span::styled(" [d] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Diff   ", theme.fg_style()),
        Span::styled(" [b/l] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Backup/Local   ", theme.fg_style()),
        Span::styled(" [Esc/q] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Return", theme.fg_style()),
    ]);
    f.render_widget(Paragraph::new(footer_line), chunks[1]);
}
