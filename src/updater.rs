use std::time::Duration;

use log::*;
use serde::{Deserialize, Serialize};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;
use tokio::fs::OpenOptions;

use crate::{
    util::{create_conversion_counter, read_json, CFG_PATH},
    Stats, TokenLock,
};

#[derive(Debug, Serialize, Deserialize)]
struct TokenBody {
    token: String,
}

pub async fn token_updater(token: TokenLock) {
    let client = reqwest::Client::new();
    loop {
        let Ok(response) = client
            .get("https://api.cider.sh/v1")
            .header("User-Agent", "Cider")
            .header("Referer", "tauri.localhost")
            .send()
            .await
        else {
            error!("Failed to get new token, keeping previous");
            return;
        };

        let Ok(serialized) = response.json::<TokenBody>().await else {
            error!("Failed to get new token, keeping previous");
            return;
        };

        *token.write().await = Some(serialized.token);

        tokio::time::sleep(Duration::from_secs(60 * 30)).await; // Sleep for 30 minutes
    }
}

pub async fn status_updater(ctx: serenity::prelude::Context) {
    let status = OnlineStatus::DoNotDisturb;
    loop {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(CFG_PATH.join("stats.json"))
            .await
            .expect("Unable to open stats.json");

        let read_stats: Stats = match read_json(&mut file).await {
            Ok(s) => s,
            Err(_) => create_conversion_counter().await,
        };

        let activity = Activity::listening(format!(
            "Cider | {} songs converted",
            read_stats.total_conversions
        ));

        ctx.set_presence(Some(activity.clone()), status).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
