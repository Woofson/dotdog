//! dmtui - TUI for dotmatrix
//!
//! Terminal user interface built with ratatui.
//! Keyboard-driven interface for managing projects with NoteDog look and feel.

use anyhow::Result;
use crate::app::{App, MainViewMode, Pane, RestoreDestination, RestoreView};
use crate::ui;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers},
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

pub fn run_tui() -> Result<()> {
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

            // 11. NoteDog Sidebar & Workspace Logical Key Dispatching
            let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);

            match (key.code, ctrl) {
                // ── Global & Modal Controls ──────────────────────────────────
                // Quit or Return to Inspector
                (KeyCode::Char('q'), false) => {
                    if app.main_view_mode != MainViewMode::Inspector {
                        app.close_main_mode();
                    } else {
                        app.should_quit = true;
                    }
                }
                (KeyCode::Esc, false) => {
                    if app.main_view_mode != MainViewMode::Inspector {
                        app.close_main_mode();
                    }
                }

                // Help Cheat Sheet & About
                (KeyCode::Char('?'), _) => {
                    app.help_scroll = 0;
                    app.show_help = true;
                }
                (KeyCode::Char('!'), _) | (KeyCode::F(2), _) => {
                    app.show_about = true;
                }

                // Fullscreen Toggle
                (KeyCode::Char('f'), false) | (KeyCode::F(11), _) | (KeyCode::Char('f'), true) => {
                    app.toggle_fullscreen();
                }

                // ── Pane & Workspace Mode Navigation ────────────────────────
                // Pane Focus Cycling
                (KeyCode::Tab, false) => {
                    app.next_pane();
                }
                (KeyCode::BackTab, _) => {
                    app.prev_pane();
                }

                // Direct Workspace Mode Selection
                (KeyCode::Char('1'), false) | (KeyCode::Char('i'), false) => {
                    app.close_main_mode();
                }
                (KeyCode::Char('2'), false) | (KeyCode::Char('+'), false) => {
                    app.open_explorer();
                }
                (KeyCode::Char('3'), false) => {
                    app.open_history();
                }

                // Vertical List Navigation
                (KeyCode::Down, _) | (KeyCode::Char('j'), false) => {
                    app.next_item(1);
                }
                (KeyCode::Up, _) | (KeyCode::Char('k'), false) => {
                    app.prev_item(1);
                }
                (KeyCode::PageDown, _) => {
                    app.next_item(PAGE_SIZE);
                }
                (KeyCode::PageUp, _) => {
                    app.prev_item(PAGE_SIZE);
                }
                (KeyCode::Home, _) => {
                    app.top_item();
                }
                (KeyCode::End, _) => {
                    app.bottom_item();
                }

                // Horizontal Navigation (Left / Right)
                (KeyCode::Left, _) | (KeyCode::Char('h'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        let previous_dir = app.browse_dir.clone();
                        if let Some(parent) = app.browse_dir.parent().map(|p| p.to_path_buf()) {
                            app.browse_dir = parent;
                            app.refresh_browse();
                            if let Some(idx) = app.browse_files.iter().position(|f| f.path == previous_dir) {
                                app.browse_list_state.select(Some(idx));
                            }
                            app.update_live_preview();
                        }
                    } else if app.main_view_mode == MainViewMode::HistoryDiff && app.restore_view == RestoreView::Files {
                        app.restore_view = RestoreView::Commits;
                        app.update_live_preview();
                    } else {
                        app.pane_left();
                    }
                }
                (KeyCode::Right, _) | (KeyCode::Char('l'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        if let Some(idx) = app.browse_list_state.selected() {
                            if let Some(file) = app.browse_files.get(idx) {
                                if file.is_dir {
                                    let path = file.path.clone();
                                    app.enter_directory(&path);
                                } else {
                                    app.pane_right();
                                }
                            }
                        }
                    } else if app.main_view_mode == MainViewMode::HistoryDiff && app.restore_view == RestoreView::Commits {
                        app.select_commit();
                        app.update_live_preview();
                    } else {
                        app.pane_right();
                    }
                }

                // ── Primary Actions ──────────────────────────────────────────
                // Enter: Contextual Primary Action
                (KeyCode::Enter, _) => {
                    match app.focused_pane {
                        Pane::Projects => {
                            app.focused_pane = Pane::Files;
                            app.update_live_preview();
                        }
                        Pane::Files => {
                            app.open_viewer();
                        }
                        Pane::Main => match app.main_view_mode {
                            MainViewMode::Inspector => {
                                app.open_viewer();
                            }
                            MainViewMode::Explorer => {
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
                            MainViewMode::HistoryDiff => {
                                if app.restore_view == RestoreView::Commits {
                                    app.select_commit();
                                    app.update_live_preview();
                                } else if app.restore_view == RestoreView::Files {
                                    app.show_restore_confirm();
                                }
                            }
                        },
                    }
                }

                // Space: Selection / Multi-Select Toggle
                (KeyCode::Char(' '), false) => {
                    if app.main_view_mode == MainViewMode::HistoryDiff {
                        app.toggle_restore_select();
                    } else if app.focused_pane == Pane::Files {
                        app.open_viewer();
                    }
                }

                // v: File Viewer
                (KeyCode::Char('v'), false) => {
                    app.open_viewer();
                }

                // a / A: Add Files mode or Add file
                (KeyCode::Char('a'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        if let Some(idx) = app.browse_list_state.selected() {
                            if let Some(file) = app.browse_files.get(idx) {
                                if !file.is_dir {
                                    let path = file.path.clone();
                                    app.add_file_to_project(&path);
                                    app.update_live_preview();
                                }
                            }
                        }
                    } else if app.main_view_mode == MainViewMode::HistoryDiff {
                        app.select_all_restore();
                    } else {
                        app.open_explorer();
                    }
                }

                // d / D: Delete / Untrack / Diffs
                (KeyCode::Char('d'), false) => {
                    if app.main_view_mode == MainViewMode::HistoryDiff {
                        app.deselect_all_restore();
                    } else if app.focused_pane == Pane::Files {
                        if let Some(file) = app.active_file() {
                            let path = file.abs_path.clone();
                            app.untrack_file(&path);
                            app.update_live_preview();
                        }
                    } else if app.focused_pane == Pane::Projects {
                        app.start_delete_project();
                    } else {
                        app.open_history();
                    }
                }
                (KeyCode::Char('D'), _) => {
                    app.start_delete_project();
                }

                // u: Explicit Untrack
                (KeyCode::Char('u'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        if let Some(idx) = app.browse_list_state.selected() {
                            if let Some(file) = app.browse_files.get(idx) {
                                if file.is_tracked() && !file.is_dir {
                                    let path = file.path.clone();
                                    app.untrack_file(&path);
                                    app.update_live_preview();
                                }
                            }
                        }
                    } else if let Some(file) = app.active_file() {
                        let path = file.abs_path.clone();
                        app.untrack_file(&path);
                        app.update_live_preview();
                    }
                }

                // n: New Project
                (KeyCode::Char('n'), _) => {
                    app.start_create_project();
                }

                // ── Backup, Sync & Git ───────────────────────────────────────
                // b: Backup commit with message
                (KeyCode::Char('b'), false) => {
                    if app.main_view_mode == MainViewMode::HistoryDiff {
                        app.view_restore_backup();
                    } else {
                        app.backup_project_with_message();
                    }
                }
                // B (Shift+B): Instant silent backup commit
                (KeyCode::Char('B'), false) => {
                    app.backup_project();
                }
                // Ctrl+B: Standalone Archive snapshot
                (KeyCode::Char('b'), true) => {
                    app.backup_project_archive();
                }

                // s: Sync project hashes against disk
                (KeyCode::Char('s'), false) => {
                    app.sync_project();
                }

                // p / P: Push / Pull from Git remote
                (KeyCode::Char('p'), false) => {
                    app.push_selected_project();
                }
                (KeyCode::Char('P'), false) => {
                    app.pull_selected_project();
                }

                // G: Set Git remote URL
                (KeyCode::Char('G'), false) => {
                    app.start_set_remote();
                }
                // g: Refresh Git remote status
                (KeyCode::Char('g'), false) => {
                    app.refresh_remote_status();
                    app.update_live_preview();
                }

                // ── Encryption & Attributes ──────────────────────────────────
                // e / E: Toggle Age encryption (file / all project files)
                (KeyCode::Char('e'), false) | (KeyCode::Char('x'), false) => {
                    app.toggle_encryption();
                    app.update_live_preview();
                }
                (KeyCode::Char('E'), false) | (KeyCode::Char('X'), false) => {
                    app.toggle_project_encryption();
                    app.update_live_preview();
                }

                // t / m: Cycle Track mode ([G] Git -> [B] Backup -> [+] Both)
                (KeyCode::Char('t'), false) | (KeyCode::Char('m'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        app.cycle_add_track_mode();
                    } else {
                        app.toggle_track_mode();
                        app.update_live_preview();
                    }
                }

                // c / C: Cleanup / Acknowledge missing files
                (KeyCode::Char('c'), false) => {
                    app.cleanup_missing_files();
                }
                (KeyCode::Char('C'), false) => {
                    app.acknowledge_missing_files();
                }

                // R: Recursive scan in Explorer
                (KeyCode::Char('R'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        app.start_recursive_preview();
                    }
                }

                // ~: Home directory in Explorer
                (KeyCode::Char('~'), false) => {
                    if app.main_view_mode == MainViewMode::Explorer {
                        if let Some(home) = dirs::home_dir() {
                            app.browse_dir = home;
                            app.refresh_browse();
                        }
                    }
                }

                // r: Refresh status
                (KeyCode::Char('r'), false) => {
                    app.refresh_projects();
                    app.message = Some(("Refreshed".to_string(), false));
                }

                _ => {}
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}
