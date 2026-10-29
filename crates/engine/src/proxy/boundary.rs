use std::collections::HashMap;

use evdev::InputEvent;

use tangible_objects_model::ABS_MT_SLOT;
use tangible_objects_model::ABS_MT_TRACKING_ID;
use tangible_objects_model::BTN_TOUCH;
use tangible_objects_model::EventType;
use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::RawEvent;
use tangible_objects_model::SlotSet;
use tangible_objects_model::TouchPointSlot;

use super::LegacyMode;
use super::stages::Stages;

/// Emit mechanism of the proxy pipeline (EVENT_PIPELINE.md).
///
/// Owns everything that describes *the virtual uinput device's view*:
/// the buffered frame, which source slots have a live contact there
/// (`emitted`), guarded-mode withheld events (`held`), retraction
/// injections and the legacy single-touch policy. The stage decides
/// *what* is suppressed; the boundary decides *how* the emitted stream
/// looks.
pub(crate) struct EmitBoundary {
    /// Events of the frame currently accumulating.
    frame: Vec<RawEvent>,
    /// Source slots with a live contact on the virtual device.
    emitted: SlotSet,
    /// Withheld events of unclassified new contacts (`Guarded` mode):
    /// a new contact's events are buffered until classification -
    /// matched slots stay held (candidate tangible), slots unmatched for
    /// `stability_gate` consecutive frames are replayed as fingers.
    held: HashMap<TouchPointSlot, HeldContact>,
    /// Policy for the legacy single-touch events (`BTN_TOUCH`, `ABS_X`,
    /// `ABS_Y`) - not slot-scoped, handled separately.
    legacy: LegacyMode,
}

/// A withheld contact in `Guarded` mode.
#[derive(Default)]
struct HeldContact {
    /// Buffered events of the contact, replayed on release.
    events: Vec<RawEvent>,
    /// Consecutive frames this contact went unmatched. Only once this
    /// reaches `stability_gate` does the contact turn out to be a finger
    /// and get released - a tangible whose pins land asynchronously
    /// stays held until the full pattern matches.
    unmatched: FrameCount,
}

impl Default for EmitBoundary {
    fn default() -> Self {
        Self::new()
    }
}

impl EmitBoundary {
    /// Creates an empty emit boundary.
    pub fn new() -> Self {
        Self {
            frame: Vec::new(),
            emitted: SlotSet::default(),
            held: HashMap::new(),
            legacy: LegacyMode::default(),
        }
    }

    /// Switches the legacy single-touch event policy (`BTN_TOUCH`,
    /// `ABS_X`, `ABS_Y`). [`LegacyMode::Auto`] (default) emits them only
    /// while a contact is live on the virtual device.
    pub fn set_legacy_mode(&mut self, legacy: LegacyMode) {
        self.legacy = legacy;
    }

    /// Buffers one source event for the current frame.
    pub fn push(&mut self, event: RawEvent) {
        self.frame.push(event);
    }

    /// Emits the buffered frame under the merged suppression view of all
    /// `stages`: drops events targeting suppressed slots and prepends
    /// `TRACKING_ID = -1` injections for suppressed slots that still have
    /// a live contact on the virtual device. Returns the complete frame
    /// to emit, ending in `SYN_REPORT`.
    ///
    /// `matched` drives the guarded-mode release decision. Mechanism
    /// behavior merges the stage policies: `TangiblesOnly` drains
    /// globally when *any* stage carries it, `Guarded` withholds when any
    /// stage is guarded (released by that stage's own gate), and a lift
    /// releases the slot on every stage.
    pub fn finish(&mut self, stages: &mut Stages, matched: &SlotSet) -> Vec<InputEvent> {
        let mode = stages.merged_mode();
        let stability_gate = stages.merged_gate();

        let mut out = Vec::with_capacity(self.frame.len() + 8);
        // Whether the clone still believes a contact is live right now -
        // captured before retraction injections and the event scan so the
        // legacy rule below can emit the BTN_TOUCH=0 down-transition.
        let had_contact = !self.emitted.is_empty();

        // TangiblesOnly: no contact ever reaches the clone. Slots still
        // live from an earlier policy get retracted once; classification
        // bookkeeping above stays alive for a consistent switch back.
        if mode == FilterMode::TangiblesOnly {
            self.frame.clear();
            self.held.clear();
            let emitted: Vec<TouchPointSlot> = self.emitted.drain().collect();
            for slot in emitted {
                out.push(input_event(ABS_MT_SLOT.as_u16(), slot.as_u32() as i32));
                out.push(input_event(ABS_MT_TRACKING_ID.as_u16(), -1));
            }
            if had_contact {
                out.push(InputEvent::new(
                    evdev::EventType(u16::from(EventType::Key)),
                    BTN_TOUCH.as_u16(),
                    0,
                ));
            }
            out.push(syn_event());
            return out;
        }

        // Withheld contacts are resolved at each frame boundary. In
        // `Guarded` a held slot that matched stays held (pending
        // tangible), one that graduated is dropped, and one accumulating
        // `stability_gate` consecutive unmatched frames is replayed as a
        // finger contact - the symmetric counterpart of the match gate.
        // Every other mode drains `held` outright - withheld events are
        // replayed so nothing is lost.
        let mut release = Vec::new();
        for (&slot, held) in self.held.iter_mut() {
            if mode != FilterMode::Guarded || stages.was_suppressed(slot) {
                release.push(slot);
            } else if matched.contains(&slot) {
                held.unmatched = FrameCount::new(0);
            } else {
                held.unmatched += 1;
                if held.unmatched >= stability_gate {
                    release.push(slot);
                }
            }
        }
        for slot in release {
            let held = self.held.remove(&slot).unwrap_or_default();
            if mode == FilterMode::Guarded && stages.was_suppressed(slot) {
                continue; // pending tangible: drop the withheld events
            }
            for raw in held.events {
                if raw.event_type == EventType::Abs && raw.code == ABS_MT_TRACKING_ID {
                    if raw.value == -1 {
                        self.emitted.remove(&slot);
                    } else {
                        self.emitted.insert(slot);
                    }
                }
                out.push(InputEvent::from(&raw));
            }
        }

        // Retract contacts that were emitted before their slot got
        // classified as a tangible.
        let retract: Vec<TouchPointSlot> = stages
            .suppressed_slots()
            .filter(|slot| self.emitted.contains(slot))
            .collect();
        for slot in retract {
            out.push(input_event(ABS_MT_SLOT.as_u16(), slot.as_u32() as i32));
            out.push(input_event(ABS_MT_TRACKING_ID.as_u16(), -1));
            self.emitted.remove(&slot);
        }

        let mut current_slot: Option<TouchPointSlot> = None;
        // Legacy single-touch events (ABS_X, ABS_Y, BTN_TOUCH) are not
        // slot-scoped - they are collected and emitted or dropped at
        // frame end depending on `legacy` and the emitted contact set.
        let mut legacy_events = Vec::new();
        for raw in self.frame.drain(..) {
            if raw.is_syn_report() {
                continue;
            }
            if raw.event_type == EventType::Abs {
                // Multitouch axes live in the ABS_MT_* range 0x2f..=0x3e
                // (SLOT, TOOL, POSITION_*, TRACKING_ID, PRESSURE, ...).
                // Legacy axes (ABS_X=0, ABS_Y=1) are slot-independent.
                let is_mt = (ABS_MT_SLOT.as_u16()..=0x3e).contains(&raw.code.as_u16());
                if !is_mt {
                    legacy_events.push(raw);
                    continue;
                }
                if raw.code == ABS_MT_SLOT {
                    current_slot = (raw.value >= 0).then(|| TouchPointSlot::new(raw.value as u32));
                }
                // A lifting contact ends the suppression - the slot becomes
                // free for a new tracking contact. Its own -1 event is
                // still dropped; the retraction injection above already
                // closed the contact on the virtual side.
                if raw.code == ABS_MT_TRACKING_ID
                    && raw.value == -1
                    && let Some(slot) = current_slot
                {
                    stages.release_slot(slot);
                }
                if current_slot.is_some_and(|slot| stages.suppresses(slot)) {
                    continue;
                }
                if mode == FilterMode::Guarded {
                    // The slot this event applies to: the SLOT selector
                    // event names its own target, every other event
                    // belongs to the current slot.
                    let target = if raw.code == ABS_MT_SLOT {
                        (raw.value >= 0).then(|| TouchPointSlot::new(raw.value as u32))
                    } else {
                        current_slot
                    };
                    if let Some(slot) = target {
                        // A fresh contact (new TRACKING_ID on a slot that
                        // is neither live on the clone nor suppressed) is
                        // withheld until classified; events of an already
                        // held slot keep accumulating.
                        let starts_contact = raw.code == ABS_MT_TRACKING_ID
                            && raw.value >= 0
                            && !self.emitted.contains(&slot);
                        if self.held.contains_key(&slot) || starts_contact {
                            self.held.entry(slot).or_default().events.push(raw);
                            continue;
                        }
                    }
                }
                if raw.code == ABS_MT_TRACKING_ID
                    && let Some(slot) = current_slot
                {
                    if raw.value == -1 {
                        self.emitted.remove(&slot);
                    } else {
                        self.emitted.insert(slot);
                    }
                }
            } else if raw.event_type == EventType::Key && raw.code == BTN_TOUCH {
                legacy_events.push(raw);
                continue;
            }
            out.push(InputEvent::from(&raw));
        }

        // Legacy single-touch events mirror "at least one active contact" -
        // emitting them with an empty virtual device would fabricate a
        // phantom tap. The down-transition (`had_contact` → none) must
        // still emit so the clone's BTN_TOUCH=0 reaches the system.
        let emit_legacy = match self.legacy {
            LegacyMode::Always => true,
            LegacyMode::Never => false,
            LegacyMode::Auto => !self.emitted.is_empty() || had_contact,
        };
        if emit_legacy {
            out.extend(legacy_events.iter().map(InputEvent::from));
        }

        out.push(syn_event());
        out
    }
}

/// Builds the `SYN_REPORT` event closing an emitted frame.
fn syn_event() -> InputEvent {
    InputEvent::from(&RawEvent::syn_report())
}

/// Builds an `EV_ABS` input event for a fixed code/value pair.
fn input_event(code: u16, value: i32) -> InputEvent {
    InputEvent::from(&RawEvent::new(EventType::Abs, code, value))
}
