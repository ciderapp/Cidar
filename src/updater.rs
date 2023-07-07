use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{Store, TokenLock};

#[derive(Debug, Serialize, Deserialize)]
struct TokenBody {
    token: String,
}

pub async fn token_updater(token: TokenLock) {
    let client = reqwest::Client::new();
    loop {
        let response: TokenBody = client
            .get("https://api.cider.sh/v1")
            .header("User-Agent", "Cider")
            .header("Referer", "tauri.localhost")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        *token.write().await = Some(response.token);

        tokio::time::sleep(Duration::from_secs(60 * 30)).await; // Sleep for 30 minutes
    }
}

pub async fn status_updater(ctx: serenity::prelude::Context) {
    use serenity::model::gateway::Activity;
    use serenity::model::user::OnlineStatus;
    let status = OnlineStatus::DoNotDisturb;
    loop {
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
