//! `count_badge` - the compact folded-content widget for data cards:
//! an icon plus a live `x / y` count. The badge is display-only - no
//! child widget claims the press, so the card's own gestures handle
//! it: `unfold_on_click` unfolds on tap and `WholeCardFolded` drags
//! the folded card (a `Button` here would claim the sequence and
//! break both).

use gtk4::Box as GtkBox;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::prelude::*;

/// Builds a folded-content badge: `icon_name` centered over a smaller
/// count label. Returns the badge widget (for
/// `CardConfig::folded_content`) and the label the caller refreshes
/// from its state ticks.
pub(crate) fn count_badge(icon_name: &str, tooltip: &str) -> (GtkBox, Label) {
    let badge = GtkBox::new(Orientation::Vertical, 0);
    badge.set_tooltip_text(Some(tooltip));
    let icon = Image::from_icon_name(icon_name);
    icon.set_pixel_size(36);
    icon.set_hexpand(true);
    icon.set_vexpand(true);
    badge.append(&icon);
    let count = Label::new(None);
    count.add_css_class("caption");
    badge.append(&count);
    (badge, count)
}
