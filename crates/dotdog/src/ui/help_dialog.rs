//! Interactive Help and Keybinding Cheat Sheet modal for dotmatrix TUI

use crate::ui::dialogs::centered_rect;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

pub fn render_help_modal(f: &mut Frame, area: Rect, scroll_y: usize, theme: &Theme) {
    let popup_area = centered_rect(80, 80, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " ❓ DOT MATRIX HELP & CHEAT SHEET [↑/↓/k/j Scroll] ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let help_lines = vec![
        Line::from(vec![Span::styled("NAVIGATION & PANES (NOTEDOG SIDEBAR)", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Tab / Shift+Tab   Cycle focus across panes (Projects → Files → Main Workspace)")]),
        Line::from(vec![Span::raw("  ← / → or h / l    Move focus between Sidebar and Main (or directory navigation)")]),
        Line::from(vec![Span::raw("  ↑ / ↓ or k / j    Navigate items in the currently focused pane")]),
        Line::from(vec![Span::raw("  PageUp / PageDown Fast scroll list, inspector, or diffs")]),
        Line::from(vec![Span::raw("  Home / End        Jump to the top / bottom of active list")]),
        Line::from(vec![Span::raw("  1                 Return to Live Project & File Inspector")]),
        Line::from(vec![Span::raw("  2 / +             Open embedded File Explorer to add files to active project")]),
        Line::from(vec![Span::raw("  3 / d / H         Open Revisions, Line Diffs & Restore")]),
        Line::from(""),
        Line::from(vec![Span::styled("PROJECT & FILE ACTIONS", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter             Focus files / Open viewer / Enter folder in Explorer")]),
        Line::from(vec![Span::raw("  b                 Backup project commit with message popup")]),
        Line::from(vec![Span::raw("  B (Shift+B)       Instant silent backup commit")]),
        Line::from(vec![Span::raw("  Ctrl+B            Create standalone Archive snapshot (tar.gz, zip, 7z)")]),
        Line::from(vec![Span::raw("  s                 Sync project file hashes against disk")]),
        Line::from(vec![Span::raw("  e / E             Toggle Age encryption (active file / ALL files in project)")]),
        Line::from(vec![Span::raw("  t                 Cycle Track Mode: [G] Git → [B] Backup → [+] Both")]),
        Line::from(vec![Span::raw("  u / d             Untrack file from project (in Files pane)")]),
        Line::from(vec![Span::raw("  n                 Create a new Project")]),
        Line::from(vec![Span::raw("  D                 Delete selected Project (with confirmation)")]),
        Line::from(vec![Span::raw("  c / C             Clean missing files / Acknowledge missing files")]),
        Line::from(vec![Span::raw("  G / g             Set Git remote URL / Refresh remote status")]),
        Line::from(vec![Span::raw("  p / P             Git Push / Git Pull from remote repository")]),
        Line::from(vec![Span::raw("  r                 Refresh project scan and disk state")]),
        Line::from(""),
        Line::from(vec![Span::styled("FILE EXPLORER MODE (2 / +)", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter / → / l     Enter directory or add file to active project")]),
        Line::from(vec![Span::raw("  ← / h / Backspace Go to parent directory")]),
        Line::from(vec![Span::raw("  a                 Add selected file to active project")]),
        Line::from(vec![Span::raw("  R                 Recursive folder scan & interactive batch add")]),
        Line::from(vec![Span::raw("  t                 Cycle default track mode for added files")]),
        Line::from(vec![Span::raw("  u                 Untrack selected file")]),
        Line::from(vec![Span::raw("  ~                 Jump directly to home directory")]),
        Line::from(vec![Span::raw("  Esc / q           Return to Live Inspector")]),
        Line::from(""),
        Line::from(vec![Span::styled("REVISIONS & DIFFS MODE (3 / d)", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter             Select commit / restore selected files")]),
        Line::from(vec![Span::raw("  Space             Toggle multi-select for restore files")]),
        Line::from(vec![Span::raw("  a / d             Select all / Deselect all restore files")]),
        Line::from(vec![Span::raw("  b                 View backup file content")]),
        Line::from(vec![Span::raw("  l                 View local file content")]),
        Line::from(vec![Span::raw("  d                 View line-by-line diff between backup and local")]),
        Line::from(vec![Span::raw("  ← / h / Backspace Go back from files to commits list")]),
        Line::from(vec![Span::raw("  Esc / q           Return to Live Inspector")]),
        Line::from(""),
        Line::from(vec![Span::styled("FILE VIEWER & PREVIEWS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  v                 Open file content in Syntax-Highlighted Viewer")]),
        Line::from(vec![Span::raw("  f / F11           Toggle Fullscreen mode for file viewer / preview")]),
        Line::from(vec![Span::raw("  n                 Toggle line numbers in viewer")]),
        Line::from(vec![Span::raw("  g / G             Jump to top / bottom of viewer content")]),
        Line::from(""),
        Line::from(vec![Span::styled("GLOBAL SHORTCUTS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  ?                 Toggle this Help & Shortcut cheat sheet modal")]),
        Line::from(vec![Span::raw("  ! / F2            Open About Dot Matrix dialog")]),
        Line::from(vec![Span::raw("  q / Esc           Close modal / Return to Inspector / Quit Dot Matrix")]),
        Line::from(""),
        Line::from(vec![Span::styled("CONFIG & THEMES LOCATION", Style::default().fg(Color::DarkGray))]),
        Line::from(vec![Span::raw("  Config:  ~/.config/dotmatrix/config.toml & manifest.toml")]),
        Line::from(vec![Span::raw("  Themes:  ~/.config/dotmatrix/themes/<theme>.toml")]),
        Line::from(""),
        Line::from(vec![Span::styled("Press [Esc], [q], or [?] to close this help window.", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
    ];

    let paragraph = Paragraph::new(help_lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    f.render_widget(paragraph, popup_area);
}
