use std::time::Duration;

use serde::{Deserialize, Serialize};

use serenity::model::gateway::Activity;
use serenity::model::user::OnlineStatus;

use crate::{Store, TokenLock, util};

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
            .await else {
                eprintln!("Failed to get new token, keeping previous");
                return
            };

        let Ok(serialized) = response
            .json::<TokenBody>()
            .await else {
                eprintln!("Failed to get new token, keeping previous");
                return
            };

        *token.write().await = Some(serialized.token);

        tokio::time::sleep(Duration::from_secs(60 * 30)).await; // Sleep for 30 minutes
    }
}

pub async fn status_updater(ctx: serenity::prelude::Context) {
    let status = OnlineStatus::DoNotDisturb;
    loop {
        // Assure we have connection to the DB
        if crate::DB.health().await.is_err() {
            eprintln!("Connection to database lost, retrying...");
            let _ = crate::DB.invalidate().await;
            util::connect_to_db().await;
            tokio::time::sleep(Duration::from_secs(10)).await;
            break;
        }

        let read_store: Store = crate::DB
            .select(("stats", "conversions"))
            .await
            .unwrap_or(Store::default());
        let activity = Activity::listening(format!(
            "Cider | {} songs converted",
            read_store.total_conversions
        ));
        ctx.set_presence(Some(activity.clone()), status).await;
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
