use std::collections::HashMap;
use std::collections::HashSet;

use tangible_objects_model::FilterMode;
use tangible_objects_model::FrameCount;
use tangible_objects_model::TouchPointSlot;

/// Frames a suppressed slot survives without a renewed match - mirrors the
/// tracker's expiry hysteresis so a transient match loss (tilt, flaky pin)
/// does not briefly leak the contact to the operating system.
pub(crate) const SUPPRESSION_HYSTERESIS: FrameCount = FrameCount::new(30);

/// Default for [`TangibleStage::set_mode`]'s stability parameter:
/// consecutive frames a slot must be matched before suppression starts.
/// A one-frame phantom match (fingers coincidentally forming a profile)
/// never enters suppression - combined with the hysteresis, which would
/// otherwise hold the wrongly-suppressed contact for ~250 ms.
/// FILTER_PROXY_VIRTUAL.md, "Mitigation 2".
const DEFAULT_STABILITY_GATE: FrameCount = FrameCount::new(2);

/// Suppression policy of a tangible stage (EVENT_PIPELINE.md).
///
/// Owns the *decision* which slots count as suppressed this frame -
/// stability gate, hysteresis bookkeeping and the active [`FilterMode`] -
/// while [`super::EmitBoundary`] owns the mechanics (dropping, retraction,
/// guarded withholding). `classify` is the stage's per-frame entry point;
/// the boundary then consumes [`Self::suppresses`]/[`Self::suppressed_slots`]
/// to shape the emitted stream.
#[derive(Debug)]
pub(crate) struct TangibleStage {
    /// Suppressed slots mapped to frames elapsed since their last match.
    /// Resets on every renewed match; expiry releases the slot.
    suppressed: HashMap<TouchPointSlot, FrameCount>,
    /// Slots matched consecutively but not yet past the stability gate.
    /// Resets when a slot misses a frame; graduates into `suppressed`
    /// after `stability_gate` consecutive matches.
    pending: HashMap<TouchPointSlot, FrameCount>,
    /// Consecutive matched frames required before suppression starts
    /// (`Gated`/`Guarded`; `Immediate` ignores it).
    stability_gate: FrameCount,
    /// Active suppression policy.
    mode: FilterMode,
}

impl TangibleStage {
    /// Creates a stage with the default policy ([`FilterMode::Guarded`],
    /// [`DEFAULT_STABILITY_GATE`]).
    pub fn new() -> Self {
        Self {
            suppressed: HashMap::new(),
            pending: HashMap::new(),
            stability_gate: DEFAULT_STABILITY_GATE,
            mode: FilterMode::default(),
        }
    }

    /// Switches the suppression policy (`SetFilterMode` control command).
    /// `stability` replaces the gate when `Some`; `None` keeps the current
    /// value. State from the previous mode is kept: existing suppressions
    /// stay suppressed, held contacts stay held until their next
    /// evaluation.
    pub fn set_mode(&mut self, mode: FilterMode, stability: Option<FrameCount>) {
        self.mode = mode;
        if let Some(gate) = stability {
            self.stability_gate = gate;
        }
    }

    /// The active suppression policy.
    pub fn mode(&self) -> FilterMode {
        self.mode
    }

    /// Consecutive matched frames required before suppression starts.
    pub fn stability_gate(&self) -> FrameCount {
        self.stability_gate
    }

    /// Whether `slot` is suppressed under the active policy - the check
    /// the emit boundary applies per event (`Off` never suppresses).
    pub fn suppresses(&self, slot: TouchPointSlot) -> bool {
        self.mode != FilterMode::Off && self.suppressed.contains_key(&slot)
    }

    /// Whether `slot` is in the suppressed set regardless of the mode
    /// flag - used by the boundary's guarded-release logic, which already
    /// knows the policy.
    pub fn was_suppressed(&self, slot: TouchPointSlot) -> bool {
        self.suppressed.contains_key(&slot)
    }

    /// All currently suppressed slots - the boundary iterates them for
    /// retraction injections.
    pub fn suppressed_slots(&self) -> impl Iterator<Item = TouchPointSlot> + '_ {
        self.suppressed.keys().copied()
    }

    /// Releases a slot whose tracking contact ended - a lifting contact
    /// frees the slot for a new tangible.
    pub fn release(&mut self, slot: TouchPointSlot) {
        self.suppressed.remove(&slot);
    }

    /// Advances classification by one frame: `matched` carries the slots
    /// of this frame's exact matches; they reset their hysteresis
    /// counter. Slots in earlier matches stay suppressed until their
    /// contact ends or [`SUPPRESSION_HYSTERESIS`] expires.
    ///
    /// Bookkeeping per policy:
    /// - `Immediate` suppresses on the first matched frame.
    /// - `Gated`/`Guarded` count consecutive matches in `pending` and
    ///   graduate into `suppressed` after `stability_gate`.
    /// - `Off` still tracks contacts (consistent state on re-enable)
    ///   but never enters suppression.
    pub fn classify(&mut self, matched: &HashSet<TouchPointSlot>) {
        match self.mode {
            FilterMode::Immediate => {
                for &slot in matched {
                    self.suppressed.insert(slot, FrameCount::new(0));
                }
            }
            FilterMode::Gated | FilterMode::Guarded => {
                self.pending.retain(|slot, _| matched.contains(slot));
                for &slot in matched {
                    if let std::collections::hash_map::Entry::Occupied(mut suppressed) =
                        self.suppressed.entry(slot)
                    {
                        *suppressed.get_mut() = FrameCount::new(0);
                    } else {
                        *self.pending.entry(slot).or_insert(FrameCount::new(0)) += 1;
                    }
                }
                let graduated: Vec<TouchPointSlot> = self
                    .pending
                    .iter()
                    .filter(|(_, count)| **count >= self.stability_gate)
                    .map(|(&slot, _)| slot)
                    .collect();
                for slot in graduated {
                    self.pending.remove(&slot);
                    self.suppressed.insert(slot, FrameCount::new(0));
                }
            }
            FilterMode::Off | FilterMode::TangiblesOnly => {}
        }
        // Age every suppression; expired entries are released.
        self.suppressed.retain(|_, age| {
            *age += 1;
            *age <= SUPPRESSION_HYSTERESIS
        });
    }
}

impl Default for TangibleStage {
    fn default() -> Self {
        Self::new()
    }
}
