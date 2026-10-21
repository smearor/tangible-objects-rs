//! Output helpers - `--json` emits the typed payload verbatim;
//! otherwise rows render as a `tabled` table.

use serde::Serialize;
use tabled::builder::Builder;
use tabled::settings::Style;

/// Prints `value` pretty-JSON when `json` is set, else hands the same
/// JSON to the `lines` formatter for a compact textual form.
pub fn emit<T: Serialize>(json: bool, value: &T, lines: impl Fn(&serde_json::Value)) {
    let value = serde_json::to_value(value).unwrap_or(serde_json::Value::Null);
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        );
    } else {
        lines(&value);
    }
}

/// Prints a `tabled` table of `headers` + `row(item)` mappings -
/// `--json` bypasses rendering and prints the typed list.
pub fn emit_table<T: Serialize>(
    json: bool,
    value: &[T],
    headers: &[&str],
    row: impl Fn(&T) -> Vec<String>,
) {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        );
        return;
    }
    let mut builder = Builder::new();
    builder.push_record(headers.iter().copied());
    for item in value {
        builder.push_record(row(item));
    }
    let mut table = builder.build();
    table.with(Style::rounded());
    println!("{table}");
}

/// Prints a single event line for `watch` (kind + the interesting bits).
pub fn event_line(event: &tangible_objects_model::TangibleEvent) -> String {
    let kind = serde_json::to_value(event.kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{:?}", event.kind));
    let mut parts = vec![kind];
    if let Some(t) = &event.tangible {
        parts.push(format!(
            "id={} {} ({:.1},{:.1})mm",
            t.id.as_u32(),
            t.pattern_name.as_str(),
            t.center.x,
            t.center.y
        ));
    }
    if let Some(name) = &event.name {
        parts.push(format!("name={name}"));
    }
    if let Some(detail) = &event.detail {
        parts.push(detail.clone());
    }
    if let Some(skipped) = event.skipped {
        parts.push(format!("skipped={skipped}"));
    }
    parts.join(" ")
}
