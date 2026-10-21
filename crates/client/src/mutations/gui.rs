//! `setGuiVisible` mutation - show/hide the daemon-embedded GUI.

use serde_json::json;

use crate::ClientError;
use crate::GraphQLClient;

impl GraphQLClient {
    /// Shows or hides the daemon's embedded GUI window
    /// (`setGuiVisible`) - ignored by headless daemons.
    pub async fn set_gui_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call(
            "setGuiVisible",
            "mutation($visible: Boolean!) { setGuiVisible(visible: $visible) }",
            json!({ "visible": visible }),
        )
        .await
    }
}
