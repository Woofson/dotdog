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
        Line::from(vec![Span::styled("NAVIGATION & TABS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Tab / Shift+Tab   Cycle between Projects, Add Files, and Restore tabs")]),
        Line::from(vec![Span::raw("  1 / 2 / 3         Direct jump to Projects (1), Add Files (2), Restore (3)")]),
        Line::from(vec![Span::raw("  ↑ / ↓ or k / j    Navigate list items vertically in active panel")]),
        Line::from(vec![Span::raw("  PageUp / PageDown Fast scroll lists, preview, or help window")]),
        Line::from(vec![Span::raw("  Home / End        Jump to the top / bottom of active list")]),
        Line::from(""),
        Line::from(vec![Span::styled("PROJECT MANAGEMENT (PROJECTS TAB)", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter / → / l     Expand / collapse project file tree")]),
        Line::from(vec![Span::raw("  ← / h             Collapse project file tree")]),
        Line::from(vec![Span::raw("  n / Ctrl+N        Create a new Project")]),
        Line::from(vec![Span::raw("  D / d / Ctrl+D    Delete selected Project (with confirmation dialog)")]),
        Line::from(vec![Span::raw("  r                 Refresh project scan and drift status")]),
        Line::from(vec![Span::raw("  S                 Save manifest and index state immediately")]),
        Line::from(""),
        Line::from(vec![Span::styled("BACKUP, SYNC & RESTORE", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  a                 Backup project with custom commit message popup")]),
        Line::from(vec![Span::raw("  A                 Silent incremental backup (no popup)")]),
        Line::from(vec![Span::raw("  b                 Create Archive snapshot (tar.gz, zip, or 7z)")]),
        Line::from(vec![Span::raw("  s                 Sync project files with store")]),
        Line::from(vec![Span::raw("  c                 Clean up missing source files from project")]),
        Line::from(vec![Span::raw("  C                 Acknowledge missing files (keep tracked, mute warning)")]),
        Line::from(""),
        Line::from(vec![Span::styled("TRACK MODES & ENCRYPTION", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  m / M             Cycle Track Mode: [G] Git → [B] Backup → [+] Both")]),
        Line::from(vec![Span::raw("  x                 Toggle age Encryption for selected file")]),
        Line::from(vec![Span::raw("  X                 Toggle age Encryption for ALL files in project")]),
        Line::from(vec![Span::raw("  u                 Session Unlock for encrypted project files")]),
        Line::from(""),
        Line::from(vec![Span::styled("GIT REMOTE OPERATIONS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  g                 Refresh Git remote status (ahead / behind / synced)")]),
        Line::from(vec![Span::raw("  G                 Set Git remote repository URL")]),
        Line::from(vec![Span::raw("  p                 Push commits to Git remote")]),
        Line::from(vec![Span::raw("  P                 Pull changes from Git remote")]),
        Line::from(""),
        Line::from(vec![Span::styled("FILE BROWSER (ADD FILES TAB)", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter / → / l     Enter directory or add file to target project")]),
        Line::from(vec![Span::raw("  ← / h / Backspace Go to parent directory")]),
        Line::from(vec![Span::raw("  a                 Add selected file to target project")]),
        Line::from(vec![Span::raw("  R                 Recursive folder scan & interactive batch add")]),
        Line::from(vec![Span::raw("  p                 Cycle active target project")]),
        Line::from(vec![Span::raw("  t                 Cycle default track mode for added files")]),
        Line::from(vec![Span::raw("  u                 Untrack selected file from projects")]),
        Line::from(vec![Span::raw("  ~                 Jump directly to home directory")]),
        Line::from(""),
        Line::from(vec![Span::styled("RESTORE & DIFF TAB", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  Enter             Select backup project / commit / restore files")]),
        Line::from(vec![Span::raw("  Space             Toggle multi-select for restore files")]),
        Line::from(vec![Span::raw("  a / d             Select all / Deselect all restore files")]),
        Line::from(vec![Span::raw("  b                 View backup file content")]),
        Line::from(vec![Span::raw("  l                 View local file content")]),
        Line::from(vec![Span::raw("  d                 View line-by-line diff between backup and local")]),
        Line::from(vec![Span::raw("  ← / h / Backspace Go back to commits or projects list")]),
        Line::from(""),
        Line::from(vec![Span::styled("FILE VIEWER & PREVIEWS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  v                 Open file content in Syntax-Highlighted Viewer")]),
        Line::from(vec![Span::raw("  f / F11           Toggle Fullscreen mode for file viewer / preview")]),
        Line::from(vec![Span::raw("  n                 Toggle line numbers in viewer")]),
        Line::from(vec![Span::raw("  g / G             Jump to top / bottom of viewer content")]),
        Line::from(""),
        Line::from(vec![Span::styled("GLOBAL SHORTCUTS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::raw("  ?                 Toggle this Help & Shortcut cheat sheet modal")]),
        Line::from(vec![Span::raw("  ! / F2 / Ctrl+A   Open About Dot Matrix dialog")]),
        Line::from(vec![Span::raw("  q / Esc           Back / Close popup / Quit Dot Matrix")]),
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
