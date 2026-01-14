//! Application state

use anyhow::Result;

use crate::commands;

/// Application state enum
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppState {
    Menu,
    Exiting,
}

/// Menu item
pub struct MenuItem {
    pub name: &'static str,
    pub description: &'static str,
    pub shortcut: char,
}

/// Main application
pub struct App {
    pub state: AppState,
    pub selection: usize,
    pub menu_items: Vec<MenuItem>,
    pub selected_action: Option<Box<dyn FnOnce() -> Result<()>>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            state: AppState::Menu,
            selection: 0,
            menu_items: vec![
                MenuItem {
                    name: "Clean",
                    description: "Free up disk space by cleaning caches",
                    shortcut: '1',
                },
                MenuItem {
                    name: "Analyze",
                    description: "Explore disk usage visually",
                    shortcut: '2',
                },
                MenuItem {
                    name: "Status",
                    description: "Monitor system health in real-time",
                    shortcut: '3',
                },
                MenuItem {
                    name: "Purge",
                    description: "Clean development project artifacts",
                    shortcut: '4',
                },
                MenuItem {
                    name: "Optimize",
                    description: "Run system maintenance tasks",
                    shortcut: '5',
                },
            ],
            selected_action: None,
        }
    }

    pub fn move_selection(&mut self, delta: i32) {
        let len = self.menu_items.len() as i32;
        let new_sel = (self.selection as i32 + delta).rem_euclid(len);
        self.selection = new_sel as usize;
    }

    pub fn select_action(&mut self) {
        self.selected_action = match self.selection {
            0 => Some(Box::new(|| commands::clean::run(false, false))),
            1 => Some(Box::new(|| {
                let home = dirs::home_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| ".".to_string());
                commands::analyze::run(home)
            })),
            2 => Some(Box::new(|| commands::status::run())),
            3 => Some(Box::new(|| commands::purge::run(None, false))),
            4 => Some(Box::new(|| commands::optimize::run(false))),
            _ => None,
        };
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
