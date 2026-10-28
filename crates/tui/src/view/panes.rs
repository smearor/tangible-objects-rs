//! Right-hand panes - the profile list and the filter/metrics pane.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::Paragraph;

use crate::app::App;

/// Renders the loaded profiles as `name pins stage owner` rows.
pub fn draw_profiles(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .profiles()
        .iter()
        .map(|info| {
            let stage = info.stage.as_ref().map(|s| s.as_str()).unwrap_or_default();
            let owner = info.owner.as_ref().map(|o| o.as_str()).unwrap_or_default();
            ListItem::new(format!(
                "{} #{} {}p {} {}",
                info.name.as_str(),
                info.pattern_id.as_u32(),
                info.num_points.as_usize(),
                stage,
                owner
            ))
        })
        .collect();
    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("profiles ({})", app.profiles().len())),
    );
    frame.render_widget(list, area);
}

/// Renders the current filter mode plus headline session metrics.
pub fn draw_metrics(frame: &mut Frame, app: &App, area: Rect) {
    let mode = app
        .filter_mode()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "?".to_string());
    let mut lines = vec![
        Line::from(format!("filter: {mode}   [f cycles]")),
        Line::from(format!(
            "stages: {}",
            app.profiles().iter().filter(|p| p.stage.is_some()).count()
        )),
    ];
    if let Some(metrics) = app.metrics() {
        lines.push(Line::from(format!(
            "frames {} tracked {}",
            metrics.frames.as_u64(),
            metrics.tracked_frames.as_u64()
        )));
        lines.push(Line::from(format!(
            "contacts {} tangibles {}",
            metrics.contacts.as_u32(),
            metrics.tangible_count.as_u64()
        )));
    }
    let pane = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("filter/metrics"),
        );
    frame.render_widget(pane, area);
}
