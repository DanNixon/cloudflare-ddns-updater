use anyhow::{anyhow, Result};
use log::{debug, info};
use matrix_sdk::{
    ruma::{
        events::{room::message::MessageEventContent, AnyMessageEventContent},
        RoomId, UserId,
    },
    Client, SyncSettings,
};
use serde::Deserialize;
use std::convert::TryFrom;

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct MatrixConfig {
    username: String,
    password: String,
    room_id: String,
    pub verbose: bool,
}

pub(crate) async fn login(config: &MatrixConfig) -> Result<Client> {
    debug! {"Logging in to Matrix..."}

    let user = UserId::try_from(config.username.clone())?;
    let client = Client::new_from_user_id(user.clone()).await?;
    client
        .login(user.localpart(), &config.password, None, None)
        .await?;

    client.sync_once(SyncSettings::new()).await?;

    info! {"Logged in to Matrix"};
    Ok(client)
}

pub(crate) async fn send_message(config: &MatrixConfig, client: &Client, msg: &str) -> Result<()> {
    let room = RoomId::try_from(config.room_id.clone())?;

    debug! {"Sending message..."}
    client
        .get_joined_room(&room)
        .unwrap()
        .send(
            AnyMessageEventContent::RoomMessage(MessageEventContent::text_plain(msg)),
            None,
        )
        .await?;

    let settings = SyncSettings::default().token(
        client
            .sync_token()
            .await
            .ok_or(anyhow! {"Could not get sync token"})?,
    );
    client.sync_once(settings).await?;

    debug! {"Message sent"}
    Ok(())
}
