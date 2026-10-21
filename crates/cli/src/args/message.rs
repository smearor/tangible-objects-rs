//! `message` args - read or push the overlay text.

/// `message` args.
#[derive(Debug, clap::Args)]
pub struct MessageArgs {
    /// Text to push via `showMessage`; omit to read the current message.
    pub text: Option<String>,
}
