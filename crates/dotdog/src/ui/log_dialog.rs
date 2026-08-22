//! Activity Log, Diagnostics & Task Management modal for dotmatrix TUI
//!
//! Provides a comprehensive 5-tab interface:
//! 1. OVERVIEW: System & repository health summary and quick recommendations
//! 2. ACTIVITY LOG: Detailed record of all session operations (what has been done)
//! 3. TO-DO LIST: Pending tasks and uncommitted/drifted items (what is to be done)
//! 4. ERRORS & FIXES: Error diagnoses, explanations of why they occurred, and step-by-step remediation guides
//! 5. STATUS GUIDE: Reference guide for status bar symbols, modes, and badges

use crate::app::{App, LogLevel};
use dmcore::FileStatus;
use crate::ui::dialogs::centered_rect;
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
    Frame,
};

/// Render the comprehensive Activity Log, Diagnostics & To-Do modal
pub fn render_log_modal(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let popup_area = centered_rect(88, 88, area);

    f.render_widget(Clear, popup_area);

    let active_tab = app.log_tab;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(theme.primary))
        .title(Span::styled(
            " 📋 ACTIVITY LOG & SYSTEM DIAGNOSTICS ",
            Style::default().fg(theme.accent).add_modifier(Modifier::BOLD),
        ));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(2), // Tab Bar
            Constraint::Min(6),    // Tab Content Body
            Constraint::Length(1), // Footer Controls
        ])
        .split(popup_area);

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Tab Bar
    // ─────────────────────────────────────────────────────────────────────────
    let activity_count = app.activity_log.len();
    let pending_tasks = app.get_pending_tasks();
    let diag_errors = app.get_diagnostic_errors();

    let tab_titles = [
        format!(" [1] OVERVIEW "),
        format!(" [2] ACTIVITY LOG ({}) ", activity_count),
        format!(" [3] TO-DO LIST ({}) ", pending_tasks.len()),
        format!(" [4] ERRORS & FIXES ({}) ", diag_errors.len()),
        format!(" [5] STATUS GUIDE "),
    ];

    let mut tab_spans = Vec::new();
    for (idx, title) in tab_titles.iter().enumerate() {
        let is_active = idx == active_tab;
        if is_active {
            tab_spans.push(Span::styled(
                title.clone(),
                Style::default().bg(theme.primary).fg(Color::Black).add_modifier(Modifier::BOLD),
            ));
        } else {
            tab_spans.push(Span::styled(
                title.clone(),
                Style::default().bg(theme.item_bg_selected).fg(theme.foreground),
            ));
        }
        if idx + 1 < tab_titles.len() {
            tab_spans.push(Span::raw("  "));
        }
    }

    let tab_bar = Paragraph::new(Line::from(tab_spans));
    f.render_widget(tab_bar, inner_chunks[0]);

    // ─────────────────────────────────────────────────────────────────────────
    // 2. Tab Content Body
    // ─────────────────────────────────────────────────────────────────────────
    let content_lines = match active_tab {
        0 => render_overview_tab(app, theme, &pending_tasks, &diag_errors),
        1 => render_activity_tab(app, theme),
        2 => render_todo_tab(app, theme, &pending_tasks),
        3 => render_errors_tab(app, theme, &diag_errors),
        _ => render_status_guide_tab(theme),
    };

    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.border));

    let content_para = Paragraph::new(content_lines)
        .block(content_block)
        .scroll((app.log_scroll as u16, 0));

    f.render_widget(content_para, inner_chunks[1]);

    // ─────────────────────────────────────────────────────────────────────────
    // 3. Footer Controls
    // ─────────────────────────────────────────────────────────────────────────
    let footer_spans = vec![
        Span::styled(" [Tab / 1-5] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Switch Tab   ", theme.fg_style()),
        Span::styled(" [↑/↓/k/j] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Scroll   ", theme.fg_style()),
        Span::styled(" [PgUp/PgDn] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Fast Scroll   ", theme.fg_style()),
        Span::styled(" [Esc / q / L] ", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled("Close", theme.fg_style()),
    ];
    let footer = Paragraph::new(Line::from(footer_spans));
    f.render_widget(footer, inner_chunks[2]);

    f.render_widget(block, popup_area);
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 1: OVERVIEW & SYSTEM HEALTH
// ─────────────────────────────────────────────────────────────────────────────
fn render_overview_tab(
    app: &App,
    theme: &Theme,
    pending: &[crate::app::PendingTask],
    errors: &[crate::app::DiagnosticError],
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    let total_projects = app.projects.len();
    let mut total_files = 0;
    let mut total_synced = 0;
    let mut total_drifted = 0;
    let mut total_missing = 0;
    let mut total_errors = 0;

    for p in &app.projects {
        total_files += p.file_count;
        total_synced += p.summary.synced;
        total_drifted += p.summary.drifted;
        total_missing += p.summary.missing;
        total_errors += p.summary.errors;
    }

    // Health Status Banner
    lines.push(Line::from(""));
    if total_errors > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ❌ ATTENTION REQUIRED: ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(" Found {} read/permission error(s) and {} missing file(s).", total_errors, total_missing),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("     Switch to tab ", theme.fg_style()),
            Span::styled("[4] ERRORS & FIXES", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" for exact error causes and step-by-step remediation instructions.", theme.fg_style()),
        ]));
    } else if total_drifted > 0 || total_missing > 0 {
        lines.push(Line::from(vec![
            Span::styled("  ⚠ PENDING ACTIONS: ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(
                format!(" {} file(s) modified on disk, {} file(s) missing from disk.", total_drifted, total_missing),
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::styled("     Switch to tab ", theme.fg_style()),
            Span::styled("[3] TO-DO LIST", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(" to review uncommitted changes and suggested backup/sync actions.", theme.fg_style()),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::styled("  ✓ SYSTEM HEALTHY: ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(
                " All tracked files exist on disk and match their recorded backup index hashes.",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ),
        ]));
    }
    lines.push(Line::from(""));

    // Global Statistics Table
    lines.push(Line::from(vec![
        Span::styled("SYSTEM TOTALS & VERIFICATION METRICS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Projects Tracked: ", theme.fg_style()),
        Span::styled(format!("{}  ", total_projects), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("│ Total Files: ", theme.fg_style()),
        Span::styled(format!("{}  ", total_files), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        Span::styled("│ Clean & Synced: ", theme.fg_style()),
        Span::styled(format!("{}  ", total_synced), Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        Span::styled("│ Drifted: ", theme.fg_style()),
        Span::styled(format!("{}  ", total_drifted), Style::default().fg(if total_drifted > 0 { Color::Yellow } else { Color::DarkGray }).add_modifier(Modifier::BOLD)),
        Span::styled("│ Missing: ", theme.fg_style()),
        Span::styled(format!("{}  ", total_missing), Style::default().fg(if total_missing > 0 { Color::LightRed } else { Color::DarkGray }).add_modifier(Modifier::BOLD)),
        Span::styled("│ Errors: ", theme.fg_style()),
        Span::styled(format!("{}", total_errors), Style::default().fg(if total_errors > 0 { Color::Red } else { Color::DarkGray }).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(""));

    // Quick Action Recommendations
    lines.push(Line::from(vec![
        Span::styled("RECOMMENDED ACTIONS", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
    ]));

    if pending.is_empty() && errors.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ✓ No urgent actions needed. Your dotfiles are fully backed up and up to date!", Style::default().fg(Color::Green)),
        ]));
    } else {
        for (i, task) in pending.iter().take(5).enumerate() {
            lines.push(Line::from(vec![
                Span::styled(format!("  {}. ", i + 1), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[{}] ", task.project), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
                Span::styled(format!("{}: ", task.title), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
                Span::styled(task.description.clone(), theme.fg_style()),
                Span::styled(format!(" ── Press {}", task.shortcut), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            ]));
        }
    }
    lines.push(Line::from(""));

    // Recent Operations Quick Log (Last 3)
    lines.push(Line::from(vec![
        Span::styled("RECENT SESSION OPERATIONS", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
    ]));
    if app.activity_log.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  (No operations recorded yet in this session.)", Style::default().fg(Color::DarkGray)),
        ]));
    } else {
        for entry in app.activity_log.iter().rev().take(4) {
            let time_str = entry.timestamp.format("%H:%M:%S").to_string();
            let (status_icon, status_color) = match entry.level {
                LogLevel::Success => ("✓", Color::Green),
                LogLevel::Warning => ("⚠", Color::Yellow),
                LogLevel::Error => ("❌", Color::Red),
                LogLevel::Info => ("ℹ", theme.primary),
            };
            let proj_tag = entry.project.as_deref().unwrap_or("DotDog");

            lines.push(Line::from(vec![
                Span::styled(format!("  {} ", status_icon), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
                Span::styled(format!("[{}] ", time_str), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("[{}] ", proj_tag), Style::default().fg(theme.accent)),
                Span::styled(entry.message.clone(), theme.fg_style()),
            ]));
        }
    }

    lines
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 2: WHAT HAS BEEN DONE (ACTIVITY HISTORY LOG)
// ─────────────────────────────────────────────────────────────────────────────
fn render_activity_tab(app: &App, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  SESSION ACTIVITY & OPERATIONS AUDIT LOG", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Chronological record of all backups, syncs, restores, git commits, and file changes in this session:", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if app.activity_log.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  (No operations performed yet. Actions like [b] Backup, [s] Sync, [p] Push, and [3] Restore will appear here.)", Style::default().fg(Color::DarkGray)),
        ]));
        return lines;
    }

    for (idx, entry) in app.activity_log.iter().rev().enumerate() {
        let time_str = entry.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

        let (level_badge, level_style) = match entry.level {
            LogLevel::Success => (" SUCCESS ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            LogLevel::Warning => (" WARNING ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            LogLevel::Error => ("  ERROR  ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            LogLevel::Info => ("  INFO   ", Style::default().bg(theme.item_bg_selected).fg(theme.primary).add_modifier(Modifier::BOLD)),
        };

        let cat_badge = match entry.category {
            crate::app::LogCategory::Backup => "[BACKUP]",
            crate::app::LogCategory::Sync => "[SYNC]",
            crate::app::LogCategory::Restore => "[RESTORE]",
            crate::app::LogCategory::Git => "[GIT]",
            crate::app::LogCategory::File => "[FILE]",
            crate::app::LogCategory::Project => "[PROJECT]",
            crate::app::LogCategory::Encryption => "[ENCRYPT]",
            crate::app::LogCategory::Scan => "[SCAN]",
        };

        let proj_str = if let Some(ref p) = entry.project {
            format!(" Project: {} │", p)
        } else {
            String::new()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  #{:02} ", idx + 1), Style::default().fg(Color::DarkGray)),
            Span::styled(level_badge, level_style),
            Span::styled(format!(" {} ", cat_badge), Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(format!("{} ", time_str), Style::default().fg(Color::DarkGray)),
            Span::styled(proj_str, Style::default().fg(theme.accent)),
            Span::styled(format!(" {}", entry.message), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ]));

        if let Some(ref details) = entry.details {
            lines.push(Line::from(vec![
                Span::raw("        └─ "),
                Span::styled(details.clone(), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
            ]));
        }

        lines.push(Line::from(""));
    }

    lines
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 3: WHAT IS TO BE DONE (PENDING TASKS & RECOMMENDATIONS)
// ─────────────────────────────────────────────────────────────────────────────
fn render_todo_tab(
    _app: &App,
    theme: &Theme,
    tasks: &[crate::app::PendingTask],
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  PENDING ACTIONS & WORK QUEUE", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  List of detected changes, uncommitted files, missing files, and pending synchronization steps:", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if tasks.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ✓ ALL UP TO DATE: Everything is clean, committed, and synchronized!", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    No pending backups, uncommitted files, or missing items found.", theme.fg_style()),
        ]));
        return lines;
    }

    for (i, task) in tasks.iter().enumerate() {
        let (cat_color, cat_bg) = match task.category {
            "ERROR RESOLUTION" => (Color::White, Color::Red),
            "MISSING FILES" => (Color::Black, Color::LightRed),
            "MODIFIED FILES" => (Color::Black, Color::Yellow),
            "NEW FILES" => (Color::Black, Color::Cyan),
            "GIT REMOTE" => (Color::Black, Color::Green),
            _ => (Color::Black, theme.primary),
        };

        lines.push(Line::from(vec![
            Span::styled(format!("  {:02}. ", i + 1), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", task.category), Style::default().bg(cat_bg).fg(cat_color).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" Project: {} ── ", task.project), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(task.title.clone(), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ]));

        lines.push(Line::from(vec![
            Span::raw("      Description: "),
            Span::styled(task.description.clone(), theme.fg_style()),
        ]));

        if !task.files.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("      Affected Files:", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            ]));
            for f in task.files.iter().take(6) {
                lines.push(Line::from(vec![
                    Span::styled("        • ", Style::default().fg(theme.primary)),
                    Span::raw(f.clone()),
                ]));
            }
            if task.files.len() > 6 {
                lines.push(Line::from(vec![
                    Span::styled(format!("        ... and {} more file(s)", task.files.len() - 6), Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                ]));
            }
        }

        lines.push(Line::from(vec![
            Span::styled("      Action: ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!("Press {}", task.shortcut), Style::default().bg(theme.item_bg_selected).fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" to perform {}", task.action_name), theme.fg_style()),
        ]));

        lines.push(Line::from(""));
    }

    lines
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 4: WHAT HAS ERRORS & HOW TO FIX THEM (DIAGNOSTICS & TROUBLESHOOTING)
// ─────────────────────────────────────────────────────────────────────────────
fn render_errors_tab(
    _app: &App,
    theme: &Theme,
    errors: &[crate::app::DiagnosticError],
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  SYSTEM DIAGNOSTICS: ROOT CAUSE ANALYSIS & REMEDIATION GUIDES", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
    ]));
    lines.push(Line::from(vec![
        Span::styled("  Detailed breakdown of what has errors, what the errors mean, and exact terminal commands to resolve them:", Style::default().fg(Color::DarkGray)),
    ]));
    lines.push(Line::from(""));

    if errors.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("  ✓ NO ERRORS DETECTED across any tracked files, permissions, or git remotes.", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from(vec![
            Span::styled("    All file checksums are valid and all paths exist on disk.", theme.fg_style()),
        ]));
        return lines;
    }

    for (i, err) in errors.iter().enumerate() {
        let (badge_text, badge_color) = match err.status {
            FileStatus::Error => ("! ERROR", Color::Red),
            FileStatus::Missing => ("✗ MISSING", Color::LightRed),
            _ => ("⚠ WARNING", Color::Yellow),
        };

        // Header
        lines.push(Line::from(vec![
            Span::styled(format!("  {:02}. ", i + 1), Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" {} ", badge_text), Style::default().bg(badge_color).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" [{}] ", err.project), Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
            Span::styled(err.target.clone(), Style::default().fg(theme.foreground).add_modifier(Modifier::BOLD)),
        ]));

        // Error message
        lines.push(Line::from(vec![
            Span::styled("      Error Detail:  ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(err.error_msg.clone(), Style::default().fg(Color::LightRed)),
        ]));

        // Meaning
        lines.push(Line::from(vec![
            Span::styled("      What It Means: ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
            Span::styled(err.meaning.clone(), theme.fg_style()),
        ]));

        // Fix steps
        lines.push(Line::from(vec![
            Span::styled("      How To Fix It: ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ]));
        for step in &err.fix_steps {
            lines.push(Line::from(vec![
                Span::styled("        ", Style::default()),
                Span::styled(step.clone(), Style::default().fg(theme.foreground)),
            ]));
        }

        lines.push(Line::from(""));
    }

    lines
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 5: STATUS GUIDE (INTEGRATED REFERENCE)
// ─────────────────────────────────────────────────────────────────────────────
fn render_status_guide_tab(theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(""),
        Line::from(vec![Span::styled("  STATUS BAR & SYSTEM INDICATORS REFERENCE", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))]),
        Line::from(vec![Span::styled("  Quick definitions of all status glyphs and mode badges:", Style::default().fg(Color::DarkGray))]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  ! N Err     ", Style::default().bg(Color::Red).fg(Color::White).add_modifier(Modifier::BOLD)),
            Span::styled(" ── N files failed SHA-256 calculation due to permission denied or read errors.", Style::default().fg(Color::Red)),
        ]),
        Line::from(vec![
            Span::styled("  ✗ N Missing ", Style::default().bg(Color::LightRed).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" ── N files in manifest.toml no longer exist on disk (deleted/renamed/moved).", Style::default().fg(Color::LightRed)),
        ]),
        Line::from(vec![
            Span::styled("  ⚠ N Drifted ", Style::default().bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" ── N files have been modified locally since the last backup revision commit.", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(vec![
            Span::styled("  + N New     ", Style::default().bg(Color::Cyan).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" ── N files freshly added to project manifest that have never been committed.", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::styled("  ✓ Synced    ", Style::default().bg(Color::Green).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" ── All tracked files exist on disk and perfectly match recorded backup hashes.", Style::default().fg(Color::Green)),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  WORKSPACE MODES", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("  NORMAL   ", Style::default().bg(theme.primary).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" [1] or [i] ── Live Project Dashboard and Syntax-Highlighted File Inspector"),
        ]),
        Line::from(vec![
            Span::styled("  EXPLORE  ", Style::default().bg(theme.secondary).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" [2] or [+] ── Embedded File Browser to add dotfiles to active project"),
        ]),
        Line::from(vec![
            Span::styled("  REVISION ", Style::default().bg(theme.accent).fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::raw(" [3] or [d] ── Commit History, Line-by-Line Diffs & Point-in-Time File Restore"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("  TRACK MODES & SECURITY", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD))]),
        Line::from(vec![
            Span::styled("  [G]  ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw("Git Only: Tracked exclusively in project's internal Git repo."),
        ]),
        Line::from(vec![
            Span::styled("  [B]  ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::raw("Backup Only: Tracked in snapshot backup archive store."),
        ]),
        Line::from(vec![
            Span::styled("  [+]  ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw("Both: Tracked in both Git and Backup systems simultaneously (Recommended)."),
        ]),
        Line::from(vec![
            Span::styled("  🔒   ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
            Span::raw("Age Encrypted: Protected with passphrase before storage (toggle [e])."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press [S] or [F3] for the standalone full-screen status indicators guide.", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        ]),
    ]
}
