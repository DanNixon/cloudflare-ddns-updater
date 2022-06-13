use anyhow::{anyhow, Result};
use matrix_sdk::{
    config::SyncSettings,
    ruma::{events::room::message::RoomMessageEventContent, RoomId, UserId},
    Client,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MatrixConfig {
    username: String,
    password: String,
    room_id: String,
    pub(crate) verbose: bool,
}

pub(crate) async fn login(config: &MatrixConfig) -> Result<Client> {
    log::debug!("Logging in to Matrix...");

    let user = UserId::parse(config.username.clone())?;
    let client = Client::builder()
        .homeserver_url(format!("https://{}", user.server_name()))
        .build()
        .await?;
    client
        .login(user.localpart(), &config.password, None, None)
        .await?;

    client.sync_once(SyncSettings::new()).await?;

    log::info!("Logged in to Matrix");
    Ok(client)
}

pub(crate) async fn send_message(config: &MatrixConfig, client: &Client, msg: &str) -> Result<()> {
    let room = RoomId::parse(config.room_id.clone())?;

    log::debug!("Sending message...");
    client
        .get_joined_room(&room)
        .unwrap()
        .send(RoomMessageEventContent::text_plain(msg), None)
        .await?;

    let settings = SyncSettings::default().token(
        client
            .sync_token()
            .await
            .ok_or_else(|| anyhow!("Could not get sync token"))?,
    );
    client.sync_once(settings).await?;

    log::debug!("Message sent");
    Ok(())
}
