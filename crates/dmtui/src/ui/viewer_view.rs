//! Syntax-highlighted File Viewer overlay and fullscreen renderer for dotmatrix TUI

use crate::app::App;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

pub fn render_viewer(f: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    if app.viewer_content.is_empty() {
        return;
    }

    f.render_widget(Clear, area);

    let border_color = if app.is_fullscreen {
        theme.active_sidebar_border
    } else {
        theme.primary
    };

    let title_prefix = if app.is_fullscreen { " ⛶ FULLSCREEN VIEWER: " } else { " 📖 " };
    let scroll_info = format!(
        "{}{} [{}/{}] ",
        title_prefix,
        app.viewer_title,
        app.viewer_scroll + 1,
        app.viewer_content.len()
    );

    // Calculate layout with top title + content area + bottom footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(area);

    let content_area = chunks[0];
    let footer_area = chunks[1];

    let visible_height = content_area.height.saturating_sub(2) as usize;
    let start = app.viewer_scroll;
    let end = (start + visible_height).min(app.viewer_content.len());
    let visible_lines = &app.viewer_content[start..end];

    let total_lines = app.viewer_content.len();
    let line_num_digits = if total_lines == 0 {
        1
    } else {
        ((total_lines as f64).log10().floor() as usize) + 1
    };
    let gutter_width = line_num_digits + 3; // digits + " │ "

    let border_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(Span::styled(
            scroll_info,
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ))
        .border_style(Style::default().fg(border_color))
        .style(theme.bg_style());

    if app.viewer_line_numbers {
        let inner_area = border_block.inner(content_area);
        f.render_widget(border_block, content_area);

        let h_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(gutter_width as u16),
                Constraint::Min(1),
            ])
            .split(inner_area);

        let gutter_area = h_chunks[0];
        let text_area = h_chunks[1];

        // Build line numbers
        let line_nums: Vec<Line> = visible_lines
            .iter()
            .enumerate()
            .map(|(idx, _)| {
                let actual_line = start + idx + 1;
                Line::from(Span::styled(
                    format!("{:>width$} │", actual_line, width = line_num_digits),
                    Style::default().fg(Color::DarkGray),
                ))
            })
            .collect();

        let gutter = Paragraph::new(line_nums).style(theme.bg_style());
        f.render_widget(gutter, gutter_area);

        // Build content lines
        let content_lines: Vec<Line> = visible_lines
            .iter()
            .map(|vl| {
                let spans: Vec<Span> = vl
                    .spans
                    .iter()
                    .map(|(text, style)| Span::styled(text.clone(), *style))
                    .collect();
                Line::from(spans)
            })
            .collect();

        let content = Paragraph::new(content_lines)
            .wrap(Wrap { trim: false })
            .style(theme.bg_style());
        f.render_widget(content, text_area);
    } else {
        let lines: Vec<Line> = visible_lines
            .iter()
            .map(|vl| {
                let spans: Vec<Span> = vl
                    .spans
                    .iter()
                    .map(|(text, style)| Span::styled(text.clone(), *style))
                    .collect();
                Line::from(spans)
            })
            .collect();

        let paragraph = Paragraph::new(lines)
            .block(border_block)
            .wrap(Wrap { trim: false })
            .style(theme.bg_style());

        f.render_widget(paragraph, content_area);
    }

    // Render footer shortcuts
    let line_num_status = if app.viewer_line_numbers { "ON" } else { "OFF" };
    let footer_spans = vec![
        Span::styled(" [↑/↓/j/k] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Scroll   ", theme.fg_style()),
        Span::styled(" [g/G] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Top/Bottom   ", theme.fg_style()),
        Span::styled(" [n] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(format!("Line# [{}]   ", line_num_status), theme.fg_style()),
        Span::styled(" [f] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Fullscreen   ", theme.fg_style()),
        Span::styled(" [q / Esc / v] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Close", theme.fg_style()),
    ];

    let footer_para = Paragraph::new(Line::from(footer_spans)).style(theme.bg_style());
    f.render_widget(footer_para, footer_area);
}
