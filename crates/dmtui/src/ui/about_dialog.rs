//! About Dialog for dotmatrix TUI

use crate::ui::dialogs::centered_rect;
use crate::theme::Theme;
use dmcore::Config;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_about_modal(f: &mut Frame, area: Rect, config: &Config, theme: &Theme) {
    let popup_area = centered_rect(68, 78, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 🤖 ABOUT DOT MATRIX ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(5),
            Constraint::Min(10),
            Constraint::Length(1),
        ])
        .split(popup_area);

    let banner = vec![
        Line::from(vec![
            Span::styled("   🤖 DOT MATRIX ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(theme.secondary)),
        ]),
        Line::from(Span::styled(
            "   \"We'll have none of that mister! How far did he get? What'd he touch?\" - Spaceballs",
            Style::default().fg(theme.foreground).add_modifier(Modifier::ITALIC),
        )),
        Line::from(Span::styled(
            "   Project compositor with per-project git versioning and content-addressed backup.",
            Style::default().fg(theme.foreground),
        )),
    ];

    let banner_para = Paragraph::new(banner);
    f.render_widget(banner_para, inner_chunks[0]);

    let owner_name = config.owner_name.as_deref().unwrap_or("Bolt J Woofson");
    let owner_web = config.owner_website.as_deref().unwrap_or("https://github.com/Woofson/dotmatrix");

    let mut details = vec![
        Line::from(vec![
            Span::styled("AUTHOR & MAINTAINER: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(owner_name, Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("REPOSITORY:          ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(owner_web, Style::default().fg(theme.secondary).add_modifier(Modifier::UNDERLINED)),
        ]),
        Line::from(vec![
            Span::styled("LICENSE:             ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled("MIT License", theme.fg_style()),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("KEY ARCHITECTURE & CAPABILITIES:", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  • Per-project isolated Git repositories, stores, and indices")]),
        Line::from(vec![Span::raw("  • 3 Track Modes: Git [G], Backup [B], or Both [+]")]),
        Line::from(vec![Span::raw("  • Per-file ChaCha20-Poly1305 / age file encryption [ENC] / 🔒")]),
        Line::from(vec![Span::raw("  • Real-time drift detection & SHA256 content verification")]),
        Line::from(vec![Span::raw("  • Live syntax-highlighted code viewer with conf.d directory assembly")]),
        Line::from(vec![Span::raw("  • Superfile & NoteDog-inspired 2-column live inspector layout")]),
        Line::from(vec![Span::raw("  • 12 NoteDog-compatible color palettes with transparent terminal support")]),
        Line::from(""),
        Line::from(vec![
            Span::styled("CONFIG PATH: ", Style::default().fg(Color::DarkGray)),
            Span::raw("~/.config/dotmatrix/config.toml"),
        ]),
        Line::from(vec![
            Span::styled("MANIFEST:    ", Style::default().fg(Color::DarkGray)),
            Span::raw("~/.config/dotmatrix/manifest.toml"),
        ]),
    ];

    if let Some(email) = &config.owner_email {
        details.insert(2, Line::from(vec![
            Span::styled("CONTACT:             ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(email, theme.fg_style()),
        ]));
    }

    let details_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border))
        .title(Span::styled(" Project Details ", theme.title_style()));

    let details_para = Paragraph::new(details).block(details_block);
    f.render_widget(details_para, inner_chunks[1]);

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(" [Esc] / [q] / [!] / [F2] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Close About Window", theme.fg_style()),
    ]));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}
