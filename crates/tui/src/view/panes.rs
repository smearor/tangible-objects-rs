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
use crate::i18n::language_loader;
use i18n_embed_fl::fl;

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
    let list = List::new(items).block(Block::default().borders(Borders::ALL).title(fl!(
        language_loader(),
        "title-profiles",
        count = (app.profiles().len() as i64)
    )));
    frame.render_widget(list, area);
}

/// Renders the current filter mode plus headline session metrics.
pub fn draw_metrics(frame: &mut Frame, app: &App, area: Rect) {
    let mode = app
        .filter_mode()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "?".to_string());
    let mut lines = vec![
        Line::from(fl!(
            language_loader(),
            "metrics-filter",
            mode = mode.clone()
        )),
        Line::from(fl!(
            language_loader(),
            "metrics-stages",
            count = (app.profiles().iter().filter(|p| p.stage.is_some()).count() as i64)
        )),
    ];
    if let Some(metrics) = app.metrics() {
        lines.push(Line::from(fl!(
            language_loader(),
            "metrics-frames",
            frames = (metrics.frames.as_u64() as i64),
            tracked = (metrics.tracked_frames.as_u64() as i64)
        )));
        lines.push(Line::from(fl!(
            language_loader(),
            "metrics-contacts",
            contacts = (metrics.contacts.as_u32() as i64),
            tangibles = (metrics.tangible_count.as_u64() as i64)
        )));
    }
    let pane = Paragraph::new(lines)
        .style(Style::default().fg(Color::Gray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(fl!(language_loader(), "title-filter-metrics")),
        );
    frame.render_widget(pane, area);
}
