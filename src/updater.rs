use std::time::Duration;

use log::*;
use serde::{Deserialize, Serialize};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

use crate::{
    util::{self, db_online},
    Store, TokenLock,
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
        // Assure we have connection to the DB
        if !db_online().await {
            error!("Connection to database lost, retrying...");
            let _ = util::connect_to_db().await;
            break;
        }

        let Ok(read_store) = crate::DB
            .select::<Option<Store>>(("stats", "conversions"))
            .await
        else {
            error!("Unable to read total conversions from the database. This can mean our connection has been severed.");
            tokio::time::sleep(Duration::from_secs(10)).await;
            continue;
        };

        let activity = Activity::listening(format!(
            "Cider | {} songs converted",
            read_store.total_conversions
        ));

        ctx.set_presence(Some(activity.clone()), status).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
