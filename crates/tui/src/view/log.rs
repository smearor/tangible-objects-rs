//! Event log pane - the last lifecycle/set/field lines.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

use crate::app::App;

/// Renders the newest log lines that fit the pane height.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let visible = area.height.saturating_sub(2) as usize;
    let lines: Vec<&String> = app.log_lines().collect();
    let items: Vec<ListItem> = lines
        .iter()
        .skip(lines.len().saturating_sub(visible))
        .map(|line| ListItem::new(line.as_str()))
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("events"));
    frame.render_widget(list, area);
}
