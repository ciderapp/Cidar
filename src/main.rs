use std::collections::HashMap;
use std::sync::Arc;
use std::task::Context;
use std::time::Duration;

use reqwest::{Url, Method};
use serde::{Serialize, Deserialize};
use serde_json::Value;
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::Timestamp;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::GuildId;
use serenity::model::prelude::Message;
use serenity::prelude::*;
use serenity::model::webhook::Webhook;

use regex::Regex;

use serde_json::json;

type TokenLock = Arc<RwLock<Option<String>>>;

struct Handler {
    client: Arc<RwLock<reqwest::Client>>,
    api: AppleMusicApi,
    url_regex: Regex,
    apple_regex: Regex,
    spotify_regex: Regex
}

struct AppleMusicApi {
    client: Arc<RwLock<reqwest::Client>>,
    developer_token: TokenLock,
}

impl AppleMusicApi {
    async fn request_endpoint(&self, method: Method, endpoint: &str) -> Value {
        // eeeeeeeeee
        self.client.read().await.request(method, format!("https://api.music.apple.com/{}", endpoint))
            .header("Authorization", format!("Bearer {}", self.developer_token.read().await.as_ref().unwrap()))
            .header("DNT", 1)
            .header("authority", "amp-api.music.apple.com")
            .header("origin", "https://beta.music.apple.com")
            .header("referer", "https://beta.music.apple.com")
            .header("sec-fetch-dest", "empty")
            .header("sec-fetch-mode", "cors")
            .header("sec-fetch-site", "same-site")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}

trait ValuePath {
    // Prevents us from having to have a hundred line structure for values.
    fn get_value_by_path(&self, path: &str) -> Option<Value>;
    fn get_vec_len_by_path(&self, path: &str) -> Option<usize>;
}

impl ValuePath for Value {
    fn get_value_by_path(&self, path: &str) -> Option<Value> {
        let mut current = self;
        for key in path.split('.') {
            if let Some(index) = key.parse::<usize>().ok() {
                current = current.get(index)?;
            } else {
                current = current.get(key)?;
            }
        }
        Some(current.clone())
    }

    fn get_vec_len_by_path(&self, path: &str) -> Option<usize> {
        let mut current = self;
        for key in path.split('.') {
            if let Some(index) = key.parse::<usize>().ok() {
                current = current.get(index)?;
            } else {
                current = current.get(key)?;
            }
        }
        Some(current.as_array().unwrap().len())
    }
}

fn wh(url: &str, w: u32, h: u32) -> String {
    url.replace("{w}", &format!("{}", w)).replace("{h}", &format!("{}", h))
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
    }

    async fn message(&self, _ctx: serenity::prelude::Context, _new_message: Message) {
        if _new_message.channel_id.0 != 1125513784384036874 {
            return;
        }

        // dont do the bot pls

        if _new_message.author.bot {
            return;
        }

        if self.url_regex.is_match(&_new_message.content) {
            println!("is url");
            let mut url = match self.url_regex.find(&_new_message.content) {
                Some(url) => url.as_str().to_string(),
                None => return,
            };

            let orig_url = url.clone();

            println!("url: {}", url);

            // Check to see it it matches either one of our regular expressions
            if !self.apple_regex.is_match(&url) && !self.spotify_regex.is_match(&url) {
                return;
            }

            if self.spotify_regex.is_match(&url) {
                let response: Value = self.client.read().await.get(format!("https://api.song.link/v1-alpha.1/links?url={}", url))
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
                println!("resp: {}", response);

                let amurl = match response.get_value_by_path("linksByPlatform.appleMusic.url") {
                    Some(url) => url,
                    None => return,
                };

                url = match amurl.as_str() {
                    Some(url) => url.to_string(),
                    None => return,
                };

                println!("amurl: {:?}", url);
            }

            let parsed_url = match Url::parse(&url) {
                Ok(p) => p,
                Err(_) => {
                    println!("failed to parse url");
                    return;
                },
            };

            let mut query: HashMap<String, String> = HashMap::new();

            // Turn the pairs into a hash map, so we can quickly index it.
            for (key, value) in parsed_url.query_pairs() {
                // Insert the key-value pair into the HashMap
                query.insert(key.to_string(), value.to_string());
            }

            let longer = url.replace("https://", "");
            let storefront: Vec<&str> = longer.split('/').collect();
            let storefront = &storefront[1];

            let mut content = _new_message.content.replace(&orig_url, "[embed]");
            if content.trim() == "[embed]" {
                content = String::from("")
            }

            let mut description: String = Default::default();
            let mut duration: Duration = Default::default();

            let mut resp: Value = Default::default();

            if url.contains("songs") || query.contains_key("i") {
                let id = match query.get("i") {
                    Some(i) => i,
                    None => return,    
                };
    
                println!("{}", id);

                resp = self.api.request_endpoint(Method::GET, &format!("v1/catalog/{}/songs/{}", storefront, &id)).await;
                description = format!("Listen to {} by {} on Cider", resp.get_value_by_path("data.0.attributes.albumName").unwrap().as_str().unwrap(), resp.get_value_by_path("data.0.attributes.artistName").unwrap().as_str().unwrap());
                duration = Duration::from_millis(resp.get_value_by_path("data.0.attributes.durationInMillis").unwrap().as_u64().unwrap_or(0));
            } else if url.contains("album") {
                resp = self.api.request_endpoint(Method::GET, &format!("v1/catalog/{}/albums/{}", storefront, parsed_url.path_segments().unwrap().last().unwrap())).await;

                let mut total_duration: u64 = 0;

                for i in 0..resp.get_vec_len_by_path("data.0.relationships.tracks.data").unwrap() {
                    total_duration += resp.get_value_by_path(&format!("data.0.relationships.tracks.data.{i}.attributes.durationInMillis")).unwrap().as_u64().unwrap();
                }

                println!("duration: {}", total_duration);

                description = format!("Listen to {} by {} on Cider", resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap(), resp.get_value_by_path("data.0.attributes.artistName").unwrap().as_str().unwrap());
                duration = Duration::from_millis(total_duration);
            } else if url.contains("playlist") {
                resp = self.api.request_endpoint(Method::GET, &format!("v1/catalog/{}/playlists/{}", storefront, parsed_url.path_segments().unwrap().last().unwrap())).await;

                let mut total_duration: u64 = 0;

                for i in 0..resp.get_vec_len_by_path("data.0.relationships.tracks.data").unwrap() {
                    total_duration += resp.get_value_by_path(&format!("data.0.relationships.tracks.data.{i}.attributes.durationInMillis")).unwrap().as_u64().unwrap();
                }

                description = format!("Listen to {} by {} on Cider", resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap(), resp.get_value_by_path("data.0.attributes.curatorName").unwrap().as_str().unwrap());
                duration = Duration::from_millis(total_duration);
            } else if url.contains("music-video") {
                resp = self.api.request_endpoint(Method::GET, &format!("v1/catalog/{}/music-video/{}", storefront, parsed_url.path_segments().unwrap().last().unwrap())).await;
                description = format!("Listen to {} by {} on Cider", resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap(), resp.get_value_by_path("data.0.attributes.artistName").unwrap().as_str().unwrap());
                duration = Duration::from_millis(resp.get_value_by_path("data.0.attributes.durationInMillis").unwrap().as_u64().unwrap_or(0));
            } else if url.contains("artist") {
                resp = self.api.request_endpoint(Method::GET, &format!("v1/catalog/{}/artists/{}", storefront, parsed_url.path_segments().unwrap().last().unwrap())).await;
                description = format!("Listen to {} on Cider", resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap());
            }
            // let wh = _new_message.channel_id.create_webhook(http, "temporary-cidar").await.unwrap();
            // wh.edit_message(http, message_id, f)
            //_new_message.channel_id.say(&_ctx.http, &format!("converted to apple id<testing>: {}", id)).await;

            _new_message.channel_id.send_message(&_ctx.http, |m| {
                m.content(content).add_embed(|e| {
                    e.title(resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap_or("N/A"))
                    .url(resp.get_value_by_path("data.0.attributes.url").unwrap().as_str().unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ"))
                    .thumbnail(wh(resp.get_value_by_path("data.0.attributes.artwork.url").unwrap().as_str().unwrap(), 512, 512))
                    .description(&description)
                    .footer(|f| {
                        f.text(format!("Shared by {} | {:?} • {}", _new_message.author.name, duration, resp.get_value_by_path("data.0.attributes.releaseDate").unwrap_or(json!(""))).as_str())
                    })
                    .timestamp(Timestamp::now())
                })
            }).await.unwrap();

            match _new_message.delete(&_ctx.http).await {
                Ok(_) => (),
                Err(_) => return,
            };
        }
    }
}

#[tokio::main]
async fn main() {
    // Get discord token
    let token = std::env::var("TOKEN").expect("Please set the TOKEN env variable");
    println!("Cidar launching");

    let developer_token: TokenLock = Default::default(); // We need this smart pointer to give to the thread that handles token updates

    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGES;

    tokio::task::spawn(token_updater(developer_token.clone()));

    // Only use 1 client for the discord stuffs, if it causes deadlocking, create a client for every request
    let discord_reqwest_client = Arc::new(RwLock::new(reqwest::Client::new()));

    let handler = Handler {
        client: discord_reqwest_client.clone(),
        api: AppleMusicApi { 
            client: discord_reqwest_client.clone(),
            developer_token: developer_token.clone() 
        },
        url_regex: Regex::new(r"(?:(?:https?|ftp)://)?[\w/\-?=%.]+\.[\w/\-&?=%.]+").unwrap(),
        apple_regex: Regex::new(r"music.apple.com/(.+[a-z](/?)+)").unwrap(),
        spotify_regex: Regex::new(r"open.spotify.com/(.+[a-z](/?)+)").unwrap(),
    };

    let mut client = Client::builder(token, intents)
        .event_handler(handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenBody {
    token: String
}

async fn token_updater(token: TokenLock) {
    let client = reqwest::Client::new();
    loop {
        let response: TokenBody = client.get("https://api.cider.sh/v1")
            .header("User-Agent", "Cider")
            .header("Referer", "tauri.localhost")
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        println!("dev_token: {}", response.token);
        *token.write().await = Some(response.token);

        tokio::time::sleep(Duration::from_secs(60 * 30)).await; // Sleep for 30 minutes 
    }
}