//! Root UI coordinator for dotmatrix TUI
//!
//! NoteDog-style Persistent Sidebar Layout with dynamic right workspace

pub mod about_dialog;
pub mod dialogs;
pub mod explorer_view;
pub mod help_dialog;
pub mod history_view;
pub mod inspector_view;
pub mod sidebar;
pub mod viewer_view;

use crate::app::{App, MainViewMode};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn render_ui(f: &mut Frame, app: &mut App) {
    let size = f.area();
    let theme = app.theme.clone();

    // 1. Wipe previous frame cells completely so no ghosts remain
    f.render_widget(Clear, size);

    // 2. Fill background if not transparent
    if !theme.transparent {
        let bg_block = Block::default().style(theme.bg_style());
        f.render_widget(bg_block, size);
    }

    let show_bar = app.config.show_help_bar;

    // Layout Split: Main 2-Pane Area (full viewport), Bottom Status Bar (1 row)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),                                // Main 2-Pane Area
            Constraint::Length(if show_bar { 1 } else { 0 }), // Bottom consolidated statusline
        ])
        .split(size);

    let workspace_area = chunks[0];
    let status_area = chunks[1];

    // ─────────────────────────────────────────────────────────────────────────
    // 1. Main Workspace: Fullscreen Viewer vs NoteDog 2-Pane Split
    // ─────────────────────────────────────────────────────────────────────────
    if app.is_fullscreen && app.viewer_visible {
        viewer_view::render_viewer(f, app, workspace_area, &theme);
    } else {
        let pane_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(34), // Left Sidebar (Projects + Files)
                Constraint::Percentage(66), // Right Workspace (Inspector / Explorer / History)
            ])
            .split(workspace_area);

        // Render Persistent Left Navigation Sidebar
        sidebar::render_sidebar(f, pane_chunks[0], app, &theme);

        // Render Dynamic Right Workspace
        match app.main_view_mode {
            MainViewMode::Inspector => {
                inspector_view::render_inspector_view(f, pane_chunks[1], app, &theme);
            }
            MainViewMode::Explorer => {
                explorer_view::render_explorer_view(f, pane_chunks[1], app, &theme);
            }
            MainViewMode::HistoryDiff => {
                history_view::render_history_view(f, pane_chunks[1], app, &theme);
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 3. Bottom Status / Help Bar
    // ─────────────────────────────────────────────────────────────────────────
    if show_bar {
        render_status_bar(f, status_area, app, &theme);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // 4. Floating Overlays & Modals
    // ─────────────────────────────────────────────────────────────────────────
    // File Viewer (non-fullscreen modal)
    if app.viewer_visible && !app.is_fullscreen && !app.restore_confirm.visible {
        let viewer_area = dialogs::centered_rect(88, 88, size);
        viewer_view::render_viewer(f, app, viewer_area, &theme);
    }

    // Help Modal
    if app.show_help {
        help_dialog::render_help_modal(f, size, app.help_scroll as usize, &theme);
    }

    // About Modal
    if app.show_about {
        about_dialog::render_about_modal(f, size, &app.config, &theme);
    }

    // Project creation dialog
    if app.creating_project {
        dialogs::render_input_dialog(
            f,
            size,
            "CREATE NEW PROJECT",
            "Enter name for the new project:",
            &app.project_input,
            Some("e.g. nvim-config, shell-scripts, ssh-keys"),
            &theme,
        );
    }

    // Git remote dialog
    if app.setting_remote {
        let proj_name = app.active_project_name().unwrap_or_default();
        dialogs::render_input_dialog(
            f,
            size,
            &format!("SET GIT REMOTE: {}", proj_name),
            "Enter Git remote repository URL:",
            &app.remote_input,
            Some("e.g. git@github.com:user/my-repo.git"),
            &theme,
        );
    }

    // Commit message dialog
    if app.entering_commit_msg {
        dialogs::render_input_dialog(
            f,
            size,
            "BACKUP COMMIT MESSAGE",
            "Enter commit message for this backup:",
            &app.commit_msg_input,
            Some("e.g. Update nvim plugins and keybindings"),
            &theme,
        );
    }

    // Delete confirmation modal
    if app.confirm_delete {
        let target = app.delete_target.as_deref().unwrap_or("project");
        dialogs::render_confirm_delete_modal(f, size, target, &theme);
    }

    // Restore confirmation modal
    if app.restore_confirm.visible {
        dialogs::render_restore_confirm_modal(
            f,
            size,
            &app.restore_confirm,
            &app.restore_files,
            &theme,
        );

        // If viewer is requested on top of restore confirm
        if app.viewer_visible {
            let viewer_area = dialogs::centered_rect(88, 88, size);
            viewer_view::render_viewer(f, app, viewer_area, &theme);
        }
    }

    // Recursive preview modal
    let target_name = app.active_project_name();
    if let Some(ref mut preview) = app.recursive_preview {
        dialogs::render_recursive_preview_modal(f, size, preview, target_name.as_deref(), &theme);
    }

    // Password prompt modal
    if app.password_prompt_visible {
        dialogs::render_passphrase_modal(
            f,
            size,
            app.password_purpose,
            &app.password_input,
            &theme,
        );
    }
}

fn render_status_bar(f: &mut Frame, area: Rect, app: &App, theme: &Theme) {
    let (mode_str, mode_color) = match app.main_view_mode {
        MainViewMode::Inspector => (" NORMAL ", theme.primary),
        MainViewMode::Explorer => (" EXPLORE ", theme.secondary),
        MainViewMode::HistoryDiff => (" REVISION ", theme.accent),
    };

    let active_project_pill = if let Some(name) = app.active_project_name() {
        let icon = app.config.icons.get_project_icon(&name);
        format!(" {} {} ", icon.trim(), name)
    } else {
        " 📦 (No Project) ".to_string()
    };

    let active_file_span = if let Some(file) = app.active_file() {
        let enc_icon = if file.encrypted { " 🔒" } else { "" };
        vec![
            Span::styled(
                format!(" 📄 {}{} ", file.path, enc_icon),
                Style::default().bg(theme.highlight_bg).fg(theme.highlight_fg).add_modifier(Modifier::BOLD),
            ),
        ]
    } else {
        vec![]
    };

    let (status_text, status_color) = if app.busy {
        (format!(" {} {} ", app.spinner(), app.busy_message), theme.primary)
    } else if let Some((ref msg, is_error)) = app.message {
        if is_error {
            (format!(" ❌ {} ", msg), Color::Red)
        } else {
            (format!(" ✓ {} ", msg), Color::Green)
        }
    } else if let Some(proj) = app.projects.get(app.active_project_idx) {
        if proj.summary.errors > 0 {
            (format!(" ! {} Err ", proj.summary.errors), Color::Red)
        } else if proj.summary.missing > 0 {
            (format!(" ✗ {} Missing ", proj.summary.missing), Color::LightRed)
        } else if proj.summary.drifted > 0 {
            (format!(" ⚠ {} Drifted ", proj.summary.drifted), Color::Yellow)
        } else if proj.summary.new > 0 {
            (format!(" + {} New ", proj.summary.new), Color::Cyan)
        } else {
            (" ✓ Synced ".to_string(), Color::Green)
        }
    } else {
        (String::new(), theme.foreground)
    };

    let mut line_spans = vec![
        Span::styled(
            mode_str,
            Style::default().bg(mode_color).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            active_project_pill,
            Style::default().bg(theme.item_bg_selected).fg(theme.foreground).add_modifier(Modifier::BOLD),
        ),
    ];
    line_spans.extend(active_file_span);
    if !status_text.is_empty() {
        line_spans.push(Span::styled(
            status_text,
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ));
    }
    line_spans.push(Span::raw(" "));
    line_spans.extend(vec![
        Span::styled(" [Tab] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Pane  ", theme.fg_style()),
        Span::styled(" [+] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Add  ", theme.fg_style()),
        Span::styled(" [b] ", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled("Backup  ", theme.fg_style()),
        Span::styled(" [d] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Diffs  ", theme.fg_style()),
        Span::styled(" [s] ", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("Sync  ", theme.fg_style()),
        Span::styled(" [e] ", Style::default().fg(theme.encrypted_tag).add_modifier(Modifier::BOLD)),
        Span::styled("Encrypt  ", theme.fg_style()),
        Span::styled(" [?] ", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled("Help", theme.fg_style()),
    ]);

    let bar = Paragraph::new(Line::from(line_spans)).style(theme.bg_style());
    f.render_widget(bar, area);
}
