//! Rendering - a `Canvas` pane drawing touches as dots and tangibles
//! as cross markers plus a status line below.

use ratatui::Frame;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::text::Line;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Points;

use crate::app::App;

/// Renders the current snapshot.
pub fn draw(frame: &mut Frame, app: &App) {
    let (touches, tangibles, bounds, status) = app.state();
    let (mm_x, mm_y) = bounds;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1)])
        .split(frame.area());

    let touch_points: Vec<(f64, f64)> = touches.iter().map(|t| (t.pos.x, t.pos.y)).collect();
    let figure_points: Vec<(f64, f64)> =
        tangibles.iter().map(|t| (t.center.x, t.center.y)).collect();

    let canvas = Canvas::default()
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("surface {mm_x:.0}x{mm_y:.0} mm")),
        )
        .marker(Marker::Braille)
        .x_bounds([0.0, mm_x])
        .y_bounds([0.0, mm_y])
        .paint(move |ctx| {
            ctx.draw(&Points {
                coords: &touch_points,
                color: Color::White,
            });
            ctx.draw(&Points {
                coords: &figure_points,
                color: Color::Cyan,
            });
            ctx.layer();
            for (i, t) in tangibles.iter().enumerate() {
                ctx.print(
                    t.center.x + 4.0,
                    t.center.y,
                    format!("{} {}", i + 1, t.pattern_name.as_str()),
                );
            }
        });
    frame.render_widget(canvas, chunks[0]);
    frame.render_widget(Paragraph::new(Line::from(status.to_string())), chunks[1]);
}
