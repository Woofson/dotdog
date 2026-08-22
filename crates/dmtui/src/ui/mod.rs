//! Root UI coordinator for dotmatrix TUI
//!
//! Brings NoteDog look and feel, multi-pane layouts, and streamlined workflow to dotmatrix.

pub mod about_dialog;
pub mod browse_view;
pub mod dialogs;
pub mod help_dialog;
pub mod projects_view;
pub mod restore_view;
pub mod viewer_view;

use crate::app::{App, Mode};
use crate::theme::Theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs},
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

    // Layout Split: Top Tabs (3 rows), Main Content, Bottom Status Bar (1 row)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                             // Tabs
            Constraint::Min(8),                                // Main content area
            Constraint::Length(if show_bar { 1 } else { 0 }), // Bottom status / help bar
        ])
        .split(size);

    let tabs_area = chunks[0];
    let main_area = chunks[1];
    let status_area = chunks[2];

    // 2. Render Top Tabs
    let tab_titles = vec![
        " 📦 PROJECTS ",
        " 📂 ADD FILES ",
        " 🔄 RESTORE & DIFF ",
    ];

    let titles: Vec<Line> = tab_titles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_active = i == app.mode.index();
            let style = if is_active {
                theme.tab_active_style()
            } else {
                theme.tab_inactive_style()
            };
            Line::from(Span::styled(*t, style))
        })
        .collect();

    let tabs_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(theme.border_style())
        .title(Span::styled(
            format!(" 🤖 DOT MATRIX v{} ", env!("CARGO_PKG_VERSION")),
            theme.sidebar_title_style(),
        ));

    let tabs = Tabs::new(titles)
        .block(tabs_block)
        .select(app.mode.index())
        .highlight_style(theme.tab_active_style());

    f.render_widget(tabs, tabs_area);

    // 3. Render Main Content (or Fullscreen Viewer if active)
    if app.is_fullscreen && app.viewer_visible {
        viewer_view::render_viewer(f, app, main_area, &theme);
    } else {
        match app.mode {
            Mode::Projects => projects_view::render_projects_view(f, main_area, app, &theme),
            Mode::Add => browse_view::render_browse_view(f, main_area, app, &theme),
            Mode::Restore => restore_view::render_restore_view(f, main_area, app, &theme),
        }
    }

    // 4. Render Bottom Status / Help Bar
    if show_bar {
        render_status_bar(f, status_area, app, &theme);
    }

    // 5. Render Floating Overlays & Modals
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
        let proj_name = app.selected_project_name().unwrap_or_default();
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
    if let Some(ref mut preview) = app.recursive_preview {
        let target_name = app.target_project.as_deref();
        dialogs::render_recursive_preview_modal(f, size, preview, target_name, &theme);
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
    let (mode_str, mode_color) = if app.is_fullscreen {
        (" FULLSCREEN ", theme.accent)
    } else {
        match app.mode {
            Mode::Projects => (" PROJECTS ", theme.primary),
            Mode::Add => (" BROWSE & ADD ", theme.secondary),
            Mode::Restore => (" RESTORE ", theme.accent),
        }
    };

    let active_crumb = match app.mode {
        Mode::Projects => {
            if let Some(name) = app.selected_project_name() {
                format!(" 📦 {} ", name)
            } else {
                " 📦 All Projects ".to_string()
            }
        }
        Mode::Add => {
            if let Some(target) = &app.target_project {
                format!(" 🎯 → 📦 {} ", target)
            } else {
                " 📂 File Explorer ".to_string()
            }
        }
        Mode::Restore => {
            if let Some(name) = &app.selected_backup_project {
                format!(" 🔄 {} ", name)
            } else {
                " 🔄 Backup Repositories ".to_string()
            }
        }
    };

    let status_text = if app.busy {
        format!(" {} {} ", app.spinner(), app.busy_message)
    } else if let Some((ref msg, is_error)) = app.message {
        if is_error {
            format!(" ❌ {} ", msg)
        } else {
            format!(" ✓ {} ", msg)
        }
    } else {
        String::new()
    };

    let status_color = if app.busy {
        Color::Yellow
    } else if let Some((_, is_error)) = app.message {
        if is_error { Color::LightRed } else { Color::Green }
    } else {
        theme.foreground
    };

    let status_spans = vec![
        Span::styled(
            mode_str,
            Style::default().bg(mode_color).fg(Color::Black).add_modifier(Modifier::BOLD),
        ),
        Span::styled(active_crumb, Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(
            status_text,
            Style::default().fg(status_color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme.border)),
        Span::styled("[Tab]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" Mode  ", theme.fg_style()),
        Span::styled("[Enter]", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" Open  ", theme.fg_style()),
        Span::styled("[a]", Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)),
        Span::styled(" Backup  ", theme.fg_style()),
        Span::styled("[v]", Style::default().fg(theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" View  ", theme.fg_style()),
        Span::styled("[?]", Style::default().fg(theme.secondary).add_modifier(Modifier::BOLD)),
        Span::styled(" Help  ", theme.fg_style()),
        Span::styled("[q]", Style::default().fg(theme.border).add_modifier(Modifier::BOLD)),
        Span::styled(" Quit", theme.fg_style()),
    ];

    let paragraph = Paragraph::new(Line::from(status_spans)).style(theme.bg_style());
    f.render_widget(Clear, area);
    f.render_widget(paragraph, area);
}
