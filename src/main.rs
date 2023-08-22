use std::collections::HashMap;
use std::sync::Arc;

use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::application::component::ButtonStyle;
use serenity::model::gateway::Ready;
use serenity::model::prelude::command::Command;
use serenity::model::prelude::{Interaction, InteractionResponseType, Message};
use serenity::model::Timestamp;
use serenity::prelude::*;

use regex::Regex;
use surrealdb::engine::remote::ws::Client;
use surrealdb::Surreal;

use log::*;

mod api;
mod commands;
mod conversion;
mod updater;
mod util;
mod vpath;

use vpath::ValuePath;

use crate::util::Cache;

type TokenLock = Arc<RwLock<Option<String>>>;
const DEBUG_CHANNEL: u64 = 1133927653074796555;

struct Handler {
    client: Arc<RwLock<reqwest::Client>>,
    api: api::AppleMusicApi,
    cache: Cache,
    url_regex: Regex,
    apple_regex: Regex,
    spotify_regex: Regex,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected", ready.user.name);
        tokio::task::spawn(updater::status_updater(ctx.clone()));

        // Setup commands
        let _ = Command::create_global_application_command(&ctx.http, |command| {
            commands::about::register(command)
        })
        .await;

        let _ = Command::create_global_application_command(&ctx.http, |command| {
            commands::convert::register(command)
        })
        .await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            // Only allow the debug channel in debug mode.
            #[cfg(debug_assertions)]
            if command.channel_id.0 != DEBUG_CHANNEL {
                return;
            }

            // In release builds, make sure to exclude the debug channel.
            #[cfg(not(debug_assertions))]
            if command.channel_id.0 == DEBUG_CHANNEL {
                return;
            }

            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("processing..."))
                })
                .await;

            let content = match command.data.name.as_str() {
                "about" => commands::about::run(&command.data.options),
                "convert" => commands::convert::run(
                    &command.data.options,
                    &self.api,
                    &self.url_regex,
                    &self.cache,
                )
                .await
                .unwrap_or_else(|err| err.to_string()),
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |response| response.content(content))
                .await
            {
                warn!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn message(&self, ctx: serenity::prelude::Context, mut new_message: Message) {
        // Only allow the debug channel in debug mode.
        #[cfg(debug_assertions)]
        if new_message.channel_id.0 != DEBUG_CHANNEL {
            return;
        }

        // In release builds, make sure to exclude the debug channel.
        #[cfg(not(debug_assertions))]
        if new_message.channel_id.0 == DEBUG_CHANNEL {
            return;
        }

        // dont do the bot pls, deleting these next 3 lines of code will cause the entire bot to implode
        if new_message.author.bot {
            return;
        }

        if self.url_regex.is_match(&new_message.content) {
            let mut url = match self.url_regex.find(&new_message.content) {
                Some(url) => url.as_str().to_string(),
                None => return,
            };

            // Check to see it it matches either one of our regular expressions
            if !self.apple_regex.is_match(&url) && !self.spotify_regex.is_match(&url) {
                return;
            }

            if self.spotify_regex.is_match(&url) {
                // nifty trick to avoid panics using let-else statements
                // and add some context to the error, even if it's fugly

                let Ok(response) = self
                    .client
                    .read()
                    .await
                    .get(format!("https://api.song.link/v1-alpha.1/links?url={url}"))
                    .send()
                    .await
                else {
                    warn!("failed to send request to song.link api");
                    return;
                };

                let Ok(serialized) = response.json::<Value>().await else {
                    warn!("failed to serialize response from song.link api");
                    return;
                };

                let Some(amurl) = serialized.get_value_by_path("linksByPlatform.appleMusic.url")
                else {
                    warn!("failed to get apple music link from song.link");
                    return;
                };

                url = match amurl.as_str() {
                    Some(url) => url.to_string(),
                    None => return,
                };
            }

            let Ok(parsed_url) = Url::parse(&url) else {
                warn!("failed to parse url");
                return;
            };

            let mut query: HashMap<String, String> = HashMap::new();

            // Turn the pairs into a hash map, so we can quickly index it.
            for (key, value) in parsed_url.query_pairs() {
                // Insert the key-value pair into the HashMap
                query.insert(key.to_string(), value.to_string());
            }

            let longer = url.replace("https://", "");

            let items = longer.split('/').collect::<Vec<&str>>();
            let Some(storefront) = items.get(1) else {
                warn!("Unable to obtain storefront from URL");
                return;
            };

            let Some(information) = conversion::get_information(
                &self.api,
                &parsed_url,
                storefront,
                &query,
                &new_message,
            )
            .await
            else {
                warn!("Unable to obtain informaiton for the embed");
                return;
            };

            let modded = url.replace("https://", "");

            let play_link = format!("https://cider.sh/p?{}", modded);
            let view_link = format!("https://cider.sh/o?{}", modded);

            let Ok(_) = new_message
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(information.title)
                            .url(information.url)
                            .thumbnail(information.artwork)
                            .description(information.description)
                            .footer(|f| f.text(information.footer))
                            .timestamp(Timestamp::now())
                    })
                    .components(|c| {
                        c.create_action_row(|r| {
                            r.create_button(|b| {
                                b.label("Play in Cider")
                                    .style(ButtonStyle::Link)
                                    .url(play_link)
                            })
                            .create_button(|b| {
                                b.label("View in Cider")
                                    .style(ButtonStyle::Link)
                                    .url(view_link)
                            })
                        })
                    })
                })
                .await
            else {
                error!("Unable to send message, ");
                return;
            };

            // Is not that important, can fail.
            let _ = new_message.suppress_embeds(&ctx.http).await;

            // Update the conversions
            util::increment_conversion(self.cache.clone()).await;
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Store {
    total_conversions: u64,
}

static DB: Surreal<Client> = Surreal::init();

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref DATABASE_IP: String =
        std::env::var("DB_IP").expect("Please set the DB_IP env variable");
    pub static ref DATABASE_PASSWORD: String =
        std::env::var("DB_PASS").expect("Please set the DB_PASS env variable");
}

#[tokio::main]
async fn main() {
    // Setup the logger
    tracing_subscriber::fmt()
        .with_env_filter("cidar=trace")
        .with_thread_names(false)
        .with_line_number(true)
        .with_file(true)
        .init();

    info!("Cidar launching");

    let token = std::env::var("TOKEN").expect("Please set the TOKEN env variable");

    info!("Starting crash governer");

    let _guard = sentry::init(("https://15cf6882a0fd0152775f80dbbf4b1c4e@o4504730117865472.ingest.sentry.io/4505693108371456", sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    info!("Connecting to SurrealDB @ {}", &*DATABASE_IP);
    let _ = util::connect_to_db().await;

    let developer_token: TokenLock = Default::default(); // We need this smart pointer to give to the thread that handles token updates

    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_INTEGRATIONS;

    tokio::task::spawn(updater::token_updater(developer_token.clone()));

    // Only use 1 client for the discord stuffs, if it causes deadlocking, create a client for every request
    let discord_reqwest_client = Arc::new(RwLock::new(reqwest::Client::new()));

    let handler = Handler {
        client: discord_reqwest_client.clone(),
        api: api::AppleMusicApi {
            client: discord_reqwest_client.clone(),
            developer_token: developer_token.clone(),
        },
        url_regex: Regex::new(r"(?:(?:https?|ftp)://)?[\w/\-?=%.]+\.[\w/\-&?=%.]+").unwrap(),
        apple_regex: Regex::new(r"music.apple.com/(.+[a-z](/?)+)").unwrap(),
        spotify_regex: Regex::new(r"open.spotify.com/(.+[a-z](/?)+)").unwrap(),
        cache: Cache::default(),
    };

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }
}
