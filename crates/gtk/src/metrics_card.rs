//! The metrics card - accumulated session counters (`SessionMetrics`)
//! and a live frame rate as a movable card on the `FloatingBoard`
//! (GUI.md §3's "Diagnostics" card).

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;

use gtk4::Box as GtkBox;
use gtk4::Grid;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::glib::ControlFlow;
use gtk4::prelude::*;
use i18n_embed_fl::fl;

use crate::language_loader;
use crate::shared_state::SharedState;

/// Builds the metrics-card content: a `label | value` grid refreshed
/// from `SharedState` every 100 ms. The fps row derives from the
/// frame counter delta between ticks - a rolling rate, not the
/// cumulative average.
pub fn create_metrics_card(state: &Arc<Mutex<SharedState>>) -> GtkBox {
    let grid = Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(4);

    let frames_value = add_row(&grid, 0, &fl!(language_loader(), "metric-frames"));
    let fps_value = add_row(&grid, 1, &fl!(language_loader(), "metric-fps"));
    let contacts_value = add_row(&grid, 2, &fl!(language_loader(), "metric-contacts"));
    let tracked_value = add_row(&grid, 3, &fl!(language_loader(), "metric-tracked"));
    let active_value = add_row(&grid, 4, &fl!(language_loader(), "metric-active"));
    let generic_value = add_row(&grid, 5, &fl!(language_loader(), "metric-generic"));
    let candidates_value = add_row(&grid, 6, &fl!(language_loader(), "metric-candidates"));
    let tangibles_value = add_row(&grid, 7, &fl!(language_loader(), "metric-tangibles"));

    let state_tick = state.clone();
    let mut last_sample: Option<(u64, Instant)> = None;
    gtk4::glib::timeout_add_local(Duration::from_millis(100), move || {
        let s = match state_tick.lock() {
            Ok(s) => s,
            Err(_) => return ControlFlow::Continue,
        };
        let metrics = &s.metrics;
        let frames = metrics.frames.as_u64();
        frames_value.set_label(&frames.to_string());
        let now = Instant::now();
        fps_value.set_label(&match last_sample.replace((frames, now)) {
            Some((prev_frames, prev_at)) => {
                let elapsed = now.duration_since(prev_at).as_secs_f64();
                format!("{:.1}", (frames - prev_frames) as f64 / elapsed.max(1e-6))
            }
            None => "–".to_string(),
        });
        contacts_value.set_label(&metrics.contacts.as_u32().to_string());
        tracked_value.set_label(&format!(
            "{} ({:.0}%)",
            metrics.tracked_frames.as_u64(),
            percent(metrics.tracked_frames.as_u64(), frames)
        ));
        active_value.set_label(&format!(
            "{} ({:.0}%)",
            metrics.active_frames.as_u64(),
            percent(metrics.active_frames.as_u64(), frames)
        ));
        generic_value.set_label(&format!(
            "{} ({:.0}%)",
            metrics.generic_frames.as_u64(),
            percent(metrics.generic_frames.as_u64(), frames)
        ));
        candidates_value.set_label(&metrics.candidates_tested.as_u64().to_string());
        tangibles_value.set_label(&metrics.tangible_count.as_u64().to_string());
        ControlFlow::Continue
    });

    let body = GtkBox::new(Orientation::Vertical, 0);
    body.append(&grid);
    body
}

/// `part` as a percentage of `total`, `0` for an empty session.
fn percent(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64 * 100.0
    }
}

/// Appends a `label | value` row to `grid` at `row`; returns the
/// value label so the refresh tick can rewrite it.
fn add_row(grid: &Grid, row: i32, name: &str) -> Label {
    let name_label = Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_opacity(0.6);
    let value = Label::new(Some("–"));
    value.set_halign(gtk4::Align::End);
    grid.attach(&name_label, 0, row, 1, 1);
    grid.attach(&value, 1, row, 1, 1);
    value
}
