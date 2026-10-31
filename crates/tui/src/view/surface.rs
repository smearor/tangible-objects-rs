//! Surface canvas - touches as dots, tangibles as labelled markers.

use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::symbols::Marker;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::canvas::Points;

use crate::app::App;
use crate::i18n::language_loader;
use i18n_embed_fl::fl;

/// Renders the braille surface canvas into `area`.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let (mm_x, mm_y) = app.bounds();
    let touch_points: Vec<(f64, f64)> = app
        .touches()
        .iter()
        .map(|touch| (touch.pos.x, touch.pos.y))
        .collect();
    let figure_points: Vec<(f64, f64)> = app
        .tangibles()
        .iter()
        .map(|tang| (tang.center.x, tang.center.y))
        .collect();
    let tangibles = app.tangibles();

    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title(fl!(
            language_loader(),
            "title-surface",
            width = (mm_x.round() as i64),
            height = (mm_y.round() as i64)
        )))
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
            for (i, tang) in tangibles.iter().enumerate() {
                ctx.print(
                    tang.center.x + 4.0,
                    tang.center.y,
                    format!("{} {}", i + 1, tang.pattern_name.as_str()),
                );
            }
        });
    frame.render_widget(canvas, area);
}
