//! Rendering - tab bar + page dispatch; panes live in sibling modules.

mod log;
mod overview;
mod panes;
mod surface;

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Tabs;

use crate::Tab;
use crate::app::App;

/// Renders the dashboard: tab bar on top, page content below, event
/// log and status line at the bottom.
pub fn draw(frame: &mut Frame, app: &App) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(6),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let tabs = Tabs::new(Tab::TITLES)
        .select(app.tab().index())
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(" | ");
    frame.render_widget(tabs, vertical[0]);

    match app.tab() {
        Tab::Overview => overview::draw(frame, app, vertical[1]),
        Tab::Live => draw_live(frame, app, vertical[1]),
    }

    log::draw(frame, app, vertical[2]);
    frame.render_widget(
        Paragraph::new(Line::from(app.status().to_string())),
        vertical[3],
    );
}

/// The live page: surface canvas left, profile/filter panes right.
fn draw_live(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let main = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(24), Constraint::Length(40)])
        .split(area);
    let right = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(6)])
        .split(main[1]);

    surface::draw(frame, app, main[0]);
    panes::draw_profiles(frame, app, right[0]);
    panes::draw_metrics(frame, app, right[1]);
}
