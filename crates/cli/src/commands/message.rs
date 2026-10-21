//! `message` - reads or pushes the user-facing GUI overlay text.

use tangible_objects_client::ClientError;
use tangible_objects_client::GraphQLClient;

use crate::args::MessageArgs;

/// `message [text]` - no text reads the current message.
pub(crate) async fn message(client: &GraphQLClient, args: &MessageArgs) -> Result<(), ClientError> {
    match &args.text {
        Some(text) => {
            client.show_message(text).await?;
        }
        None => {
            let msg = client.message().await?;
            println!("{}", msg.unwrap_or_default());
        }
    }
    Ok(())
}
