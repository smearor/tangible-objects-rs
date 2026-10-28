//! Overview tab - four columns showing what physically lies on the
//! surface: active slots, matched tangibles, occupancy fields with
//! their live counts, and the registered field sets.

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::layout::Rect;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;

use crate::app::App;

/// Renders the four occupancy columns.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    draw_slots(frame, app, columns[0]);
    draw_tangibles(frame, app, columns[1]);
    draw_fields(frame, app, columns[2]);
    draw_field_sets(frame, app, columns[3]);
}

/// Active contact slots with their calibrated position.
fn draw_slots(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .touches()
        .iter()
        .map(|touch| {
            ListItem::new(format!(
                "slot {} ({:.0},{:.0})",
                touch.slot.as_u32(),
                touch.pos.x,
                touch.pos.y
            ))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("slots ({})", app.touches().len())),
    );
    frame.render_widget(list, area);
}

/// Matched tangibles with pattern name and position.
fn draw_tangibles(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .tangibles()
        .iter()
        .map(|tang| {
            ListItem::new(format!(
                "{} #{} ({:.0},{:.0}){}",
                tang.pattern_name.as_str(),
                tang.id.as_u32(),
                tang.center.x,
                tang.center.y,
                if tang.pending { " pending" } else { "" }
            ))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("tangibles ({})", app.tangibles().len())),
    );
    frame.render_widget(list, area);
}

/// All occupancy fields with their last reported contact count.
fn draw_fields(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .field_sets()
        .iter()
        .flat_map(|set| {
            set.fields.iter().map(move |field| {
                let count = app
                    .field_count(&set.name, &field.name)
                    .map(|c| c.as_u32().to_string())
                    .unwrap_or_else(|| "-".to_string());
                ListItem::new(format!(
                    "{}.{}: {}",
                    set.name.as_str(),
                    field.name.as_str(),
                    count
                ))
            })
        })
        .collect();
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("fields"));
    frame.render_widget(list, area);
}

/// The registered field sets with stability/suppress/window info.
fn draw_field_sets(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .field_sets()
        .iter()
        .map(|set| {
            ListItem::new(format!(
                "{} ({} fields, stab {}, {}){}",
                set.name.as_str(),
                set.fields.len(),
                set.stability.as_u64(),
                if set.suppress { "suppress" } else { "info" },
                if set.window.is_some() { " +window" } else { "" }
            ))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("field sets ({})", app.field_sets().len())),
    );
    frame.render_widget(list, area);
}
