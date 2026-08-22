//! dmtui - TUI for dotmatrix
//!
//! Terminal user interface built with ratatui.
//! Keyboard-driven interface for managing projects with NoteDog look and feel.

mod app;
mod theme;
mod ui;

use anyhow::Result;
use app::{App, Mode, RestoreDestination, RestoreView};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

const PAGE_SIZE: usize = 10;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new()?;

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Save any dirty state
    app.save_if_dirty()?;

    res
}

fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::render_ui(f, app))?;

        // Poll background operations
        app.poll_operation();

        // Poll for events with timeout
        let poll_timeout = if app.busy {
            Duration::from_millis(80)
        } else {
            Duration::from_millis(250)
        };

        if !event::poll(poll_timeout)? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            // Clear transient message on keypress
            if !app.busy {
                app.message = None;
            }

            // Ignore keys while busy (except quit)
            if app.busy {
                if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                    app.should_quit = true;
                }
                continue;
            }

            // 1. Help Modal Active
            if app.show_help {
                match key.code {
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.help_scroll = app.help_scroll.saturating_add(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.help_scroll = app.help_scroll.saturating_sub(1);
                    }
                    KeyCode::PageDown => {
                        app.help_scroll = app.help_scroll.saturating_add(8);
                    }
                    KeyCode::PageUp => {
                        app.help_scroll = app.help_scroll.saturating_sub(8);
                    }
                    KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') => {
                        app.show_help = false;
                        app.help_scroll = 0;
                    }
                    _ => {}
                }
                continue;
            }

            // 2. About Modal Active
            if app.show_about {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('!') | KeyCode::F(2) => {
                        app.show_about = false;
                    }
                    _ => {
                        app.show_about = false;
                    }
                }
                continue;
            }

            // 3. File Viewer Mode Active
            if app.viewer_visible && !app.restore_confirm.visible {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                        app.close_viewer();
                        app.is_fullscreen = false;
                    }
                    KeyCode::Char('f') | KeyCode::F(11) => {
                        app.toggle_fullscreen();
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.viewer_scroll_down(1);
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.viewer_scroll_up(1);
                    }
                    KeyCode::PageDown => {
                        app.viewer_scroll_down(PAGE_SIZE);
                    }
                    KeyCode::PageUp => {
                        app.viewer_scroll_up(PAGE_SIZE);
                    }
                    KeyCode::Home | KeyCode::Char('g') => {
                        app.viewer_scroll_top();
                    }
                    KeyCode::End | KeyCode::Char('G') => {
                        app.viewer_scroll_bottom();
                    }
                    KeyCode::Char('n') => {
                        app.toggle_viewer_line_numbers();
                    }
                    _ => {}
                }
                continue;
            }

            // 4. Project Creation Input Active
            if app.creating_project {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_create_project();
                        app.update_live_preview();
                    }
                    KeyCode::Esc => {
                        app.cancel_create_project();
                    }
                    KeyCode::Backspace => {
                        app.project_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.project_input.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            // 5. Git Remote Configuration Input Active
            if app.setting_remote {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_set_remote();
                        app.update_live_preview();
                    }
                    KeyCode::Esc => {
                        app.cancel_set_remote();
                    }
                    KeyCode::Backspace => {
                        app.remote_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.remote_input.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            // 6. Custom Commit Message Input Active
            if app.entering_commit_msg {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_commit_msg();
                    }
                    KeyCode::Esc => {
                        app.cancel_commit_msg();
                    }
                    KeyCode::Backspace => {
                        app.commit_msg_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.commit_msg_input.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            // 7. Recursive Preview Modal Active
            if app.recursive_preview.is_some() {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.cancel_recursive_preview();
                    }
                    KeyCode::Enter => {
                        app.confirm_recursive_add();
                        app.update_live_preview();
                    }
                    KeyCode::PageDown => {
                        for _ in 0..PAGE_SIZE {
                            app.preview_next();
                        }
                    }
                    KeyCode::PageUp => {
                        for _ in 0..PAGE_SIZE {
                            app.preview_previous();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        app.preview_next();
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        app.preview_previous();
                    }
                    KeyCode::Char(' ') => {
                        app.toggle_preview_file();
                        app.preview_next();
                    }
                    KeyCode::Char('a') => {
                        app.toggle_all_preview_files();
                    }
                    KeyCode::Char('t') => {
                        app.toggle_preview_track_mode();
                    }
                    KeyCode::Char('T') => {
                        app.set_all_preview_track_mode();
                    }
                    _ => {}
                }
                continue;
            }

            // 8. Password Prompt Modal Active
            if app.password_prompt_visible {
                match key.code {
                    KeyCode::Enter => {
                        app.confirm_password();
                    }
                    KeyCode::Esc => {
                        app.cancel_password();
                    }
                    KeyCode::Backspace => {
                        app.password_input.pop();
                    }
                    KeyCode::Char(c) => {
                        app.password_input.push(c);
                    }
                    _ => {}
                }
                continue;
            }

            // 9. Delete Confirmation Modal Active
            if app.confirm_delete {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                        app.confirm_delete_project();
                        app.update_live_preview();
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.cancel_delete();
                    }
                    _ => {}
                }
                continue;
            }

            // 10. Restore Confirmation Modal Active
            if app.restore_confirm.visible {
                if app.viewer_visible {
                    match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.viewer_scroll = app.viewer_scroll.saturating_add(1);
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.viewer_scroll = app.viewer_scroll.saturating_sub(1);
                        }
                        KeyCode::PageDown => {
                            app.viewer_scroll = app.viewer_scroll.saturating_add(20);
                        }
                        KeyCode::PageUp => {
                            app.viewer_scroll = app.viewer_scroll.saturating_sub(20);
                        }
                        KeyCode::Char('g') | KeyCode::Home => {
                            app.viewer_scroll = 0;
                        }
                        KeyCode::Char('G') | KeyCode::End => {
                            app.viewer_scroll = app.viewer_content.len().saturating_sub(1);
                        }
                        KeyCode::Char('n') => {
                            app.toggle_viewer_line_numbers();
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('v') => {
                            app.close_restore_preview();
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.restore_confirm.entering_path {
                    match key.code {
                        KeyCode::Enter => {
                            app.restore_confirm.entering_path = false;
                        }
                        KeyCode::Esc => {
                            app.restore_confirm.entering_path = false;
                            app.restore_confirm.custom_path.clear();
                            app.restore_confirm.destination = RestoreDestination::Original;
                        }
                        KeyCode::Backspace => {
                            app.restore_confirm.custom_path.pop();
                        }
                        KeyCode::Char(c) => {
                            app.restore_confirm.custom_path.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                            app.confirm_restore();
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Char('q') => {
                            app.cancel_restore_confirm();
                        }
                        KeyCode::Char('o') | KeyCode::Char('O') => {
                            app.restore_confirm.destination = RestoreDestination::Original;
                            app.restore_confirm.entering_path = false;
                        }
                        KeyCode::Char('c') | KeyCode::Char('C') => {
                            app.restore_confirm.destination = RestoreDestination::Custom;
                            app.restore_confirm.entering_path = true;
                        }
                        KeyCode::Tab => {
                            app.toggle_restore_destination();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.restore_confirm_up();
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.restore_confirm_down();
                        }
                        KeyCode::Char('b') | KeyCode::Char('B') => {
                            app.view_restore_backup();
                        }
                        KeyCode::Char('l') | KeyCode::Char('L') => {
                            app.view_restore_local();
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            app.view_restore_diff();
                        }
                        _ => {}
                    }
                }
                continue;
            }

            // 11. Global Keybindings
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            match (key.code, ctrl) {
                (KeyCode::Char('q'), false) => app.should_quit = true,
                (KeyCode::Char('?'), _) => {
                    app.help_scroll = 0;
                    app.show_help = true;
                }
                (KeyCode::Char('!'), _) | (KeyCode::F(2), _) | (KeyCode::Char('a'), true) => {
                    app.show_about = true;
                }
                (KeyCode::Char('f'), false) | (KeyCode::F(11), _) | (KeyCode::Char('f'), true) => {
                    app.toggle_fullscreen();
                }
                (KeyCode::Tab, false) => {
                    let next = (app.mode.index() + 1) % 3;
                    app.mode = Mode::from_index(next);
                    if app.mode == Mode::Restore {
                        app.restore_view = RestoreView::Projects;
                        app.restore_selected.clear();
                        app.scan_backup_projects();
                    }
                    app.update_live_preview();
                }
                (KeyCode::BackTab, _) => {
                    let prev = (app.mode.index() + 2) % 3;
                    app.mode = Mode::from_index(prev);
                    if app.mode == Mode::Restore {
                        app.restore_view = RestoreView::Projects;
                        app.restore_selected.clear();
                        app.scan_backup_projects();
                    }
                    app.update_live_preview();
                }
                (KeyCode::Char('1'), false) => {
                    app.mode = Mode::Projects;
                    app.update_live_preview();
                }
                (KeyCode::Char('2'), false) => {
                    app.mode = Mode::Add;
                    app.update_live_preview();
                }
                (KeyCode::Char('3'), false) => {
                    app.mode = Mode::Restore;
                    app.restore_view = RestoreView::Projects;
                    app.restore_selected.clear();
                    app.scan_backup_projects();
                    app.update_live_preview();
                }
                _ => {
                    // Mode-specific key handler
                    match app.mode {
                        Mode::Projects => handle_projects_keys(app, key.code, ctrl),
                        Mode::Add => handle_add_keys(app, key.code, ctrl),
                        Mode::Restore => handle_restore_keys(app, key.code, ctrl),
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_projects_keys(app: &mut App, key: KeyCode, ctrl: bool) {
    match (key, ctrl) {
        (KeyCode::Down, _) | (KeyCode::Char('j'), false) => {
            app.projects_next(1);
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), false) => {
            app.projects_prev(1);
        }
        (KeyCode::PageDown, _) => {
            app.projects_next(PAGE_SIZE);
        }
        (KeyCode::PageUp, _) => {
            app.projects_prev(PAGE_SIZE);
        }
        (KeyCode::Home, _) => {
            app.projects_top();
        }
        (KeyCode::End, _) => {
            app.projects_bottom();
        }
        (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), false) => {
            app.toggle_selected_project();
        }
        (KeyCode::Left, _) | (KeyCode::Char('h'), false) => {
            app.collapse_selected_project();
        }
        (KeyCode::Char('a'), false) => {
            // Backup with custom commit message popup
            app.backup_project_with_message();
        }
        (KeyCode::Char('A'), false) => {
            // Silent backup
            app.backup_project();
        }
        (KeyCode::Char('b'), false) => {
            // Archive backup (zip / tar.gz / 7z)
            app.backup_project_archive();
        }
        (KeyCode::Char('s'), false) => {
            app.sync_project();
        }
        (KeyCode::Char('c'), false) => {
            app.cleanup_missing_files();
        }
        (KeyCode::Char('C'), false) => {
            app.acknowledge_missing_files();
        }
        (KeyCode::Char('r'), false) => {
            app.refresh_projects();
            app.message = Some(("Refreshed".to_string(), false));
        }
        (KeyCode::Char('n'), _) => {
            app.start_create_project();
        }
        (KeyCode::Char('D'), _) | (KeyCode::Char('d'), _) => {
            app.start_delete_project();
        }
        (KeyCode::Char('x'), false) => {
            app.toggle_encryption();
            app.update_live_preview();
        }
        (KeyCode::Char('X'), false) => {
            app.toggle_project_encryption();
            app.update_live_preview();
        }
        (KeyCode::Char('m'), false) | (KeyCode::Char('M'), false) => {
            app.toggle_track_mode();
            app.update_live_preview();
        }
        (KeyCode::Char('S'), false) => {
            app.save_and_reload();
        }
        (KeyCode::Char('g'), false) => {
            app.refresh_remote_status();
            app.update_live_preview();
        }
        (KeyCode::Char('G'), false) => {
            app.start_set_remote();
        }
        (KeyCode::Char('p'), false) => {
            app.push_selected_project();
        }
        (KeyCode::Char('P'), false) => {
            app.pull_selected_project();
        }
        (KeyCode::Char('v'), false) => {
            app.open_viewer();
        }
        _ => {}
    }
}

fn handle_add_keys(app: &mut App, key: KeyCode, ctrl: bool) {
    match (key, ctrl) {
        (KeyCode::Down, _) | (KeyCode::Char('j'), false) => {
            app.browse_next(1);
        }
        (KeyCode::Up, _) | (KeyCode::Char('k'), false) => {
            app.browse_prev(1);
        }
        (KeyCode::PageDown, _) => {
            app.browse_next(PAGE_SIZE);
        }
        (KeyCode::PageUp, _) => {
            app.browse_prev(PAGE_SIZE);
        }
        (KeyCode::Home, _) => {
            app.browse_top();
        }
        (KeyCode::End, _) => {
            app.browse_bottom();
        }
        (KeyCode::Enter, _) | (KeyCode::Right, _) | (KeyCode::Char('l'), false) => {
            if let Some(idx) = app.browse_list_state.selected() {
                if let Some(file) = app.browse_files.get(idx) {
                    let path = file.path.clone();
                    if file.is_dir {
                        app.enter_directory(&path);
                    } else {
                        app.add_file_to_project(&path);
                        app.update_live_preview();
                    }
                }
            }
        }
        (KeyCode::Left, _) | (KeyCode::Char('h'), false) | (KeyCode::Backspace, _) => {
            let previous_dir = app.browse_dir.clone();
            if let Some(parent) = app.browse_dir.parent().map(|p| p.to_path_buf()) {
                app.browse_dir = parent;
                app.refresh_browse();
                if let Some(idx) = app.browse_files.iter().position(|f| f.path == previous_dir) {
                    app.browse_list_state.select(Some(idx));
                }
                app.update_live_preview();
            }
        }
        (KeyCode::Char('a'), false) => {
            if let Some(idx) = app.browse_list_state.selected() {
                if let Some(file) = app.browse_files.get(idx) {
                    if !file.is_dir {
                        let path = file.path.clone();
                        app.add_file_to_project(&path);
                        app.update_live_preview();
                    }
                }
            }
        }
        (KeyCode::Char('~'), false) => {
            if let Some(home) = dirs::home_dir() {
                app.browse_dir = home;
                app.refresh_browse();
            }
        }
        (KeyCode::Char('p'), false) => {
            app.cycle_target_project();
            app.update_live_preview();
        }
        (KeyCode::Char('n'), _) => {
            app.start_create_project();
        }
        (KeyCode::Char('R'), false) => {
            app.start_recursive_preview();
        }
        (KeyCode::Char('t'), false) => {
            app.cycle_add_track_mode();
        }
        (KeyCode::Char('u'), false) => {
            if let Some(idx) = app.browse_list_state.selected() {
                if let Some(file) = app.browse_files.get(idx) {
                    if file.is_tracked() && !file.is_dir {
                        let path = file.path.clone();
                        app.untrack_file(&path);
                        app.update_live_preview();
                    }
                }
            }
        }
        (KeyCode::Char('v'), false) => {
            app.open_viewer();
        }
        _ => {}
    }
}

fn handle_restore_keys(app: &mut App, key: KeyCode, _ctrl: bool) {
    match app.restore_view {
        RestoreView::Projects => match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.backup_projects.is_empty() {
                    let i = app.backup_project_list_state.selected().unwrap_or(0);
                    let next = (i + 1).min(app.backup_projects.len() - 1);
                    app.backup_project_list_state.select(Some(next));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.backup_projects.is_empty() {
                    let i = app.backup_project_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(1);
                    app.backup_project_list_state.select(Some(prev));
                }
            }
            KeyCode::PageDown => {
                if !app.backup_projects.is_empty() {
                    let i = app.backup_project_list_state.selected().unwrap_or(0);
                    let next = (i + PAGE_SIZE).min(app.backup_projects.len() - 1);
                    app.backup_project_list_state.select(Some(next));
                }
            }
            KeyCode::PageUp => {
                if !app.backup_projects.is_empty() {
                    let i = app.backup_project_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(PAGE_SIZE);
                    app.backup_project_list_state.select(Some(prev));
                }
            }
            KeyCode::Home => {
                if !app.backup_projects.is_empty() {
                    app.backup_project_list_state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !app.backup_projects.is_empty() {
                    app.backup_project_list_state.select(Some(app.backup_projects.len() - 1));
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                app.select_backup_project();
            }
            KeyCode::Char('r') => {
                app.scan_backup_projects();
                app.message = Some(("Refreshed".to_string(), false));
            }
            _ => {}
        },
        RestoreView::Commits => match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.commits.is_empty() {
                    let i = app.commit_list_state.selected().unwrap_or(0);
                    let next = (i + 1).min(app.commits.len() - 1);
                    app.commit_list_state.select(Some(next));
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.commits.is_empty() {
                    let i = app.commit_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(1);
                    app.commit_list_state.select(Some(prev));
                }
            }
            KeyCode::PageDown => {
                if !app.commits.is_empty() {
                    let i = app.commit_list_state.selected().unwrap_or(0);
                    let next = (i + PAGE_SIZE).min(app.commits.len() - 1);
                    app.commit_list_state.select(Some(next));
                }
            }
            KeyCode::PageUp => {
                if !app.commits.is_empty() {
                    let i = app.commit_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(PAGE_SIZE);
                    app.commit_list_state.select(Some(prev));
                }
            }
            KeyCode::Home => {
                if !app.commits.is_empty() {
                    app.commit_list_state.select(Some(0));
                }
            }
            KeyCode::End => {
                if !app.commits.is_empty() {
                    app.commit_list_state.select(Some(app.commits.len() - 1));
                }
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                app.select_commit();
                app.update_live_preview();
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => {
                app.back_to_backup_projects();
            }
            KeyCode::Char('r') => {
                if let Some(name) = app.selected_backup_project.clone() {
                    app.load_commits_for_project(&name);
                }
                app.message = Some(("Refreshed".to_string(), false));
            }
            _ => {}
        },
        RestoreView::Files => match key {
            KeyCode::Down | KeyCode::Char('j') => {
                if !app.restore_files.is_empty() {
                    let i = app.restore_list_state.selected().unwrap_or(0);
                    let next = (i + 1).min(app.restore_files.len() - 1);
                    app.restore_list_state.select(Some(next));
                    app.update_live_preview();
                }
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !app.restore_files.is_empty() {
                    let i = app.restore_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(1);
                    app.restore_list_state.select(Some(prev));
                    app.update_live_preview();
                }
            }
            KeyCode::PageDown => {
                if !app.restore_files.is_empty() {
                    let i = app.restore_list_state.selected().unwrap_or(0);
                    let next = (i + PAGE_SIZE).min(app.restore_files.len() - 1);
                    app.restore_list_state.select(Some(next));
                    app.update_live_preview();
                }
            }
            KeyCode::PageUp => {
                if !app.restore_files.is_empty() {
                    let i = app.restore_list_state.selected().unwrap_or(0);
                    let prev = i.saturating_sub(PAGE_SIZE);
                    app.restore_list_state.select(Some(prev));
                    app.update_live_preview();
                }
            }
            KeyCode::Home => {
                if !app.restore_files.is_empty() {
                    app.restore_list_state.select(Some(0));
                    app.update_live_preview();
                }
            }
            KeyCode::End => {
                if !app.restore_files.is_empty() {
                    app.restore_list_state.select(Some(app.restore_files.len() - 1));
                    app.update_live_preview();
                }
            }
            KeyCode::Enter => {
                app.show_restore_confirm();
            }
            KeyCode::Char(' ') => {
                app.toggle_restore_select();
            }
            KeyCode::Char('v') => {
                app.open_viewer();
            }
            KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace => {
                app.back_to_commits();
            }
            KeyCode::Char('r') => {
                if let Some(commit_idx) = app.selected_commit {
                    let hash = app.commits[commit_idx].hash.clone();
                    app.load_commit_files(&hash);
                }
                app.message = Some(("Refreshed".to_string(), false));
            }
            KeyCode::Char('a') => {
                app.select_all_restore();
            }
            KeyCode::Char('d') => {
                app.deselect_all_restore();
            }
            _ => {}
        },
    }
}
