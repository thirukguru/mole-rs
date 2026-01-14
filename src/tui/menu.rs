//! Main menu rendering

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::app::App;

/// Render the main menu
pub fn render_menu(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(7),  // Header
            Constraint::Min(10),    // Menu
            Constraint::Length(3),  // Footer
        ])
        .split(f.size());

    render_header(f, chunks[0]);
    render_menu_items(f, chunks[1], app);
    render_footer(f, chunks[2]);
}

fn render_header(f: &mut Frame, area: ratatui::layout::Rect) {
    let banner = vec![
        Line::from(""),
        Line::from(Span::styled(
            "🐹 Mole-RS",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            "Deep clean and optimize your Ubuntu system",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(""),
    ];

    let header = Paragraph::new(banner)
        .alignment(Alignment::Center)
        .block(Block::default());

    f.render_widget(header, area);
}

fn render_menu_items(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    let items: Vec<ListItem> = app
        .menu_items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let is_selected = i == app.selection;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let prefix = if is_selected { "▶ " } else { "  " };
            let shortcut = format!("[{}] ", item.shortcut);

            let content = Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(shortcut, Style::default().fg(Color::Yellow)),
                Span::styled(item.name, style.add_modifier(Modifier::BOLD)),
                Span::raw("  "),
                Span::styled(item.description, Style::default().fg(Color::DarkGray)),
            ]);

            ListItem::new(content)
        })
        .collect();

    let menu = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray))
                .title(" Select an action "),
        )
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(menu, area);
}

fn render_footer(f: &mut Frame, area: ratatui::layout::Rect) {
    let help = Line::from(vec![
        Span::styled("↑↓", Style::default().fg(Color::Yellow)),
        Span::raw(" Navigate   "),
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(" Select   "),
        Span::styled("1-5", Style::default().fg(Color::Yellow)),
        Span::raw(" Quick select   "),
        Span::styled("q", Style::default().fg(Color::Yellow)),
        Span::raw(" Quit"),
    ]);

    let footer = Paragraph::new(help)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(footer, area);
}
