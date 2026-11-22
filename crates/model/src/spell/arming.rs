//! `SpellArming` - how a spell's gesture is initiated (SPELLS.md's
//! Einläutung phase): the explicit trigger that suppresses
//! ambiguity.

use serde::Deserialize;
use serde::Serialize;
use std::str::FromStr;

use crate::HandName;
use crate::PatternName;
use crate::Region;

/// How a spell is armed - every spell declares exactly one arming
/// type (`[arming]` TOML section with a `type` tag).
///
/// The armed state is what justifies suppression: while the state
/// machine is `Armed`/`Active`, the spell's contacts route to the
/// gesture tracker and never reach the virtual device.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SpellArming {
    /// A reserved surface region - contacts inside it are always
    /// gesture candidates; no field may overlap it and new
    /// tangibles never match inside, while already-tracked
    /// tangibles may be dragged through.
    ExclusiveZone {
        /// The reserved rectangle in surface millimeters.
        region: Region,
    },
    /// A small region that arms the spell when the gesture's first
    /// contact lands on it - screen corner, window edge, or a
    /// button an application registered.
    StartSpot {
        /// The arming rectangle in surface millimeters.
        region: Region,
    },
    /// A brief whole-hand tap (~5 simultaneous contacts inside a
    /// tight time window) arms a short window; the gesture itself
    /// may use fewer fingers.
    FiveFingerTap {
        /// The handprint this tap must match - `Some` binds the
        /// spell to a registered [`crate::HandProfile`] (SPELLS.md's
        /// per-user spells); `None` accepts any whole-hand tap.
        #[serde(default)]
        hand: Option<HandName>,
    },
    /// A registered tangible acts as the wand - armed while the
    /// tangible is tracked, its center pose is the stroke, lift
    /// completes the spell.
    TangibleChord {
        /// The wand's pattern - `None` accepts any tracked
        /// tangible.
        pattern: Option<PatternName>,
    },
    /// No explicit trigger - the optional fallback that uses
    /// prefix-gated defer-and-replay (SPELLS.md's pinch/rotate
    /// caveat applies).
    Free,
}

impl FromStr for SpellArming {
    type Err = toml::de::Error;

    /// Parses the `[arming]` TOML table - `type = "<kebab-kind>"`
    /// plus the fields the armed variants carry (e.g.
    /// `type = "start-spot"` + `[region]`). An empty string yields
    /// [`SpellArming::Free`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Self::Free);
        }
        toml::from_str(s)
    }
}
