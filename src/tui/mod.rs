//! Terminal UI using ratatui

mod app;
mod menu;

pub use app::App;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;
use std::io;

use app::AppState;

/// Run the interactive TUI
pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app state
    let mut app = App::new();

    // Run main loop
    let result = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    // Handle result
    if let Err(err) = result {
        eprintln!("Error: {}", err);
        return Err(err);
    }

    // Execute selected action if any
    if let Some(action) = app.selected_action.take() {
        action()?;
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| {
            match app.state {
                AppState::Menu => menu::render_menu(f, app),
                AppState::Exiting => {}
            }
        })?;

        if app.state == AppState::Exiting {
            return Ok(());
        }

        // Handle events
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => {
                            app.state = AppState::Exiting;
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.move_selection(-1);
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.move_selection(1);
                        }
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        KeyCode::Char('1') => {
                            app.selection = 0;
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        KeyCode::Char('2') => {
                            app.selection = 1;
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        KeyCode::Char('3') => {
                            app.selection = 2;
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        KeyCode::Char('4') => {
                            app.selection = 3;
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        KeyCode::Char('5') => {
                            app.selection = 4;
                            app.select_action();
                            if app.selected_action.is_some() {
                                app.state = AppState::Exiting;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}
