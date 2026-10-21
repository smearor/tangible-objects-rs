//! `set_gui_visible()` method of [`GraphQLClient`].

use crate::ClientError;
use crate::GraphQLClient;
use crate::mutations::gui::mutation::SetGuiVisible;
use crate::mutations::gui::variables::SetGuiVisibleVariables;

impl GraphQLClient {
    /// Shows or hides the daemon's embedded GUI window
    /// (`setGuiVisible`) - ignored by headless daemons.
    pub async fn set_gui_visible(&self, visible: bool) -> Result<bool, ClientError> {
        self.call_mutation::<SetGuiVisible, _, _>(
            "setGuiVisible",
            SetGuiVisibleVariables { visible },
        )
        .await
    }
}
