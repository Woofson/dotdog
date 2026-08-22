//! Status Bar and System Indicators Guide modal for dotmatrix TUI
//!
//! Provides detailed descriptions of all status glyphs, bottom status-bar badges
//! (e.g. "! 5 Err", "✗ 3 Missing", "⚠ 2 Drifted", "+ 1 New", "✓ Synced"),
//! modes, track badges, encryption status, and git synchronization states.

use crate::ui::dialogs::centered_rect;
use crate::theme::Theme;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

/// Render the Status Bar & System Indicators Guide modal
pub fn render_status_guide_modal(f: &mut Frame, area: Rect, scroll_y: usize, theme: &Theme) {
    let popup_area = centered_rect(82, 84, area);

    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 📊 STATUS BAR & SYSTEM INDICATORS GUIDE [↑/↓/k/j Scroll | Esc Close] ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let mut lines = Vec::new();

    // ── SECTION 1: BOTTOM STATUS BAR & HEALTH BADGES ─────────────────────────
    lines.push(Line::from(vec![
        Span::styled("BOTTOM STATUS BAR HEALTH INDICATORS", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::raw("The bottom status bar shows the real-time synchronization state of the active project:"),
    ]));
    lines.push(Line::from(""));

    // 1. ! N Err
    lines.push(Line::from(vec![
        Span::styled("  ! N Err  ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(" ── File Read / Permission / I/O Error", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Meaning: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("N tracked files encountered an operating system error when DotDog tried to read them."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Causes:  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("• Permission Denied (e.g. root-owned /etc files with 0600 or 0700 permissions)."),
    ]));
    lines.push(Line::from(vec![
        Span::raw("           • Broken symlinks pointing to non-existent targets."),
    ]));
    lines.push(Line::from(vec![
        Span::raw("           • File is locked exclusively by another application or drive is unmounted."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Fix:     ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Open Activity Log & Diagnostics ("),
        Span::styled("[L]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("), check permissions with "),
        Span::styled("chmod +r <path>", Style::default().fg(theme.secondary)),
        Span::raw(", or untrack with "),
        Span::styled("[u]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("."),
    ]));
    lines.push(Line::from(""));

    // 2. ✗ N Missing
    lines.push(Line::from(vec![
        Span::styled("  ✗ N Missing  ", Style::default().bg(Color::LightRed).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" ── Tracked File Missing on Disk", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Meaning: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("N files are listed in project manifest (~/.config/dotdog/manifest.toml), but don't exist on disk."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Causes:  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("File was deleted, moved, or renamed outside of DotDog."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Fix:     ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Restore from previous backup in Revisions mode ("),
        Span::styled("[3]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("), or clean up entries with "),
        Span::styled("[c]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(" / untrack with "),
        Span::styled("[u]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("."),
    ]));
    lines.push(Line::from(""));

    // 3. ⚠ N Drifted
    lines.push(Line::from(vec![
        Span::styled("  ⚠ N Drifted  ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" ── Local Modifications Detected", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Meaning: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("N tracked files exist on disk, but their SHA-256 hash differs from the last saved backup/index."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Causes:  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("You or an editor (e.g. Neovim, VSCode) modified the file since the last backup commit."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Fix:     ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Save changes into a new backup snapshot with "),
        Span::styled("[b]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" (or instant "),
        Span::styled("[B]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("), or sync hashes with "),
        Span::styled("[s]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("."),
    ]));
    lines.push(Line::from(""));

    // 4. + N New
    lines.push(Line::from(vec![
        Span::styled("  + N New  ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" ── Newly Tracked / Uncommitted Files", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Meaning: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("N files were added to the project manifest and have never been committed to backup storage."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Fix:     ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Press "),
        Span::styled("[b]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw(" to record the initial backup commit for the new file(s)."),
    ]));
    lines.push(Line::from(""));

    // 5. ✓ Synced
    lines.push(Line::from(vec![
        Span::styled("  ✓ Synced  ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" ── Fully Synchronized & Clean", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Meaning: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("All tracked files exist on disk and match the recorded SHA-256 hashes in the backup index."),
    ]));
    lines.push(Line::from(""));

    // ── SECTION 2: WORKSPACE MODES ───────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("WORKSPACE MODES (MAIN RIGHT PANE)", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  NORMAL   ", Style::default().bg(theme.primary).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" [1] or [i] ── Live Project Dashboard & Syntax-Highlighted File Inspector", theme.fg_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  EXPLORE  ", Style::default().bg(theme.secondary).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" [2] or [+] ── Embedded File Browser to add dotfiles to active project", theme.fg_style()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  REVISION ", Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
        Span::styled(" [3] or [d] ── Commit History, Line-by-Line Diffs & File Restoration", theme.fg_style()),
    ]));
    lines.push(Line::from(""));

    // ── SECTION 3: TRACK MODES & SECURITY BADGES ─────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("TRACK MODES & ENCRYPTION ATTRIBUTES", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [G]  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::styled("Git Only:      ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::raw("File is versioned exclusively in the project's internal Git repository."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [B]  ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("Backup Only:   ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::raw("File is saved in point-in-time snapshot backup archives."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  [+]  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("Both (Rec.):   ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::raw("File is tracked in both Git and Backup systems simultaneously."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  🔒   ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Age Encrypted: ", Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::raw("Protected with modern Age encryption passphrase (toggle with "),
        Span::styled("[e]", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::raw(")."),
    ]));
    lines.push(Line::from(""));

    // ── SECTION 4: GIT REMOTE STATES ─────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("GIT REMOTE SYNCHRONIZATION STATES", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Up to date ✓       ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::raw("Local Git repository matches the remote repository exactly."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Ahead of remote    ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("Local commits are ready to push to remote (press "),
        Span::styled("[p]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(")."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Behind remote      ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw("Remote repository has new commits ready to pull (press "),
        Span::styled("[P]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw(")."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Ahead and behind   ", Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)),
        Span::raw("Diverged history. Pull and merge remote changes with "),
        Span::styled("[P]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("."),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Remote unreachable ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw("Network offline, DNS failure, or SSH key authentication error."),
    ]));
    lines.push(Line::from(""));

    // ── SECTION 5: MODAL NAVIGATION ──────────────────────────────────────────
    lines.push(Line::from(vec![
        Span::styled("MODAL & LOG SHORTCUTS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  L / Shift+L / F4   ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::raw("Open Activity Log & Diagnostics Modal (History, To-Do, Errors & Fixes)"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  S / Shift+S / F3   ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::raw("Open this Status Indicators Guide Modal"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  ?                  ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::raw("Open full Keybinding & Navigation Cheat Sheet"),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Esc / q / S        ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::raw("Close this guide and return to DotDog"),
    ]));

    let paragraph = Paragraph::new(lines)
        .block(block)
        .scroll((scroll_y as u16, 0));

    f.render_widget(paragraph, popup_area);
}
