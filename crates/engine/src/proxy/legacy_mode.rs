/// Policy for the legacy single-touch events `BTN_TOUCH`, `ABS_X` and
/// `ABS_Y` in the filter proxy.
///
/// These events are not slot-scoped: the driver emits them once per frame
/// mirroring "at least one active contact". When every multitouch slot is
/// suppressed (e.g. only a tangible rests on the screen), forwarding them
/// would still signal a contact to the compositor - phantom taps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LegacyMode {
    /// Emit legacy events only when at least one contact is live on the
    /// virtual device at frame end - the physically correct mirror.
    #[default]
    Auto,
    /// Always pass legacy events through (debugging).
    Always,
    /// Never emit legacy events - modern stacks consume MT data only.
    Never,
}

impl std::fmt::Display for LegacyMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Self::Auto => "auto",
            Self::Always => "always",
            Self::Never => "never",
        };
        write!(f, "{name}")
    }
}

impl std::str::FromStr for LegacyMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            "never" => Ok(Self::Never),
            _ => Err(()),
        }
    }
}
