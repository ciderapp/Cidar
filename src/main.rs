use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use reqwest::{Method, Url};
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
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

mod api;
mod commands;
mod updater;
mod util;
mod vpath;

use vpath::ValuePath;

type TokenLock = Arc<RwLock<Option<String>>>;

struct Handler {
    client: Arc<RwLock<reqwest::Client>>,
    api: api::AppleMusicApi,
    url_regex: Regex,
    apple_regex: Regex,
    spotify_regex: Regex,
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected", ready.user.name);
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
            //let Rc<RefCell<>>
            let _ = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content("processing..."))
                })
                .await
                .unwrap();

            let content = match command.data.name.as_str() {
                "about" => commands::about::run(&command.data.options),
                "convert" => {
                    commands::convert::run(&command.data.options, &self.api, &self.url_regex)
                        .await
                        .unwrap_or_else(|err| err.to_string())
                }
                _ => "not implemented".to_string(),
            };

            if let Err(why) = command
                .edit_original_interaction_response(&ctx.http, |response| response.content(content))
                .await
            {
                println!("Cannot respond to slash command: {why}");
            }
        }
    }

    async fn message(&self, ctx: serenity::prelude::Context, mut new_message: Message) {
        // if new_message.channel_id.0 != 1125513784384036874 {
        //     return;
        // }

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
                    .await else {
                        eprintln!("failed to send request to song.link api");
                        return;
                    };

                let Ok(serialized) = response
                    .json::<Value>()
                    .await else {
                        eprintln!("failed to serialize response from song.link api");
                        return;
                    };

                let amurl = serialized
                    .get_value_by_path("linksByPlatform.appleMusic.url")
                    .unwrap();

                url = match amurl.as_str() {
                    Some(url) => url.to_string(),
                    None => return,
                };
            }

            let parsed_url = match Url::parse(&url) {
                Ok(p) => p,
                Err(_) => {
                    println!("failed to parse url");
                    return;
                }
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

            let description: String;

            let mut duration: Duration = Default::default();

            let resp: Value = Value::Null;

            let mut media = Media::default();

            if url.contains("song") || query.contains_key("i") {
                let id = match query.get("i") {
                    Some(i) => i,
                    None => parsed_url.path_segments().unwrap().last().unwrap(),
                };

                let Ok(resp) = self
                    .api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/songs/{}", storefront, &id),
                    )
                    .await else {
                        eprintln!("failed to request song {id} from the apple music api");
                        return
                    };

                media.sid = id.to_string();
                media.media_type = MediaType::Song;

                // return useless values instead of panicking
                let name = resp
                    .get_value_by_path("data.0.attributes.name")
                    .unwrap()
                    .as_str()
                    .unwrap_or("N/A")
                    .to_string();
                media.name = name;

                description = format!(
                    "Listen to {} by {} on Cider",
                    resp.get_value_by_path("data.0.attributes.albumName")
                        .unwrap()
                        .as_str()
                        .unwrap_or("N/A"),
                    resp.get_value_by_path("data.0.attributes.artistName")
                        .unwrap()
                        .as_str()
                        .unwrap_or("N/A")
                );
                duration = Duration::from_millis(
                    resp.get_value_by_path("data.0.attributes.durationInMillis")
                        .unwrap()
                        .as_u64()
                        .unwrap_or(0),
                );
            } else if url.contains("album") {
                let id = parsed_url.path_segments().unwrap().last().unwrap();
                let Ok(resp) = self
                    .api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/albums/{}", storefront, id),
                    )
                    .await else {
                        eprintln!("failed to request album {id} from the apple music api");
                        return
                    };

                media.sid = id.to_string();
                media.media_type = MediaType::Album;

                let mut total_duration: u64 = 0;

                for i in 0..resp
                    .get_vec_len_by_path("data.0.relationships.tracks.data")
                    .unwrap()
                {
                    total_duration += resp
                        .get_value_by_path(&format!(
                            "data.0.relationships.tracks.data.{i}.attributes.durationInMillis"
                        ))
                        .unwrap()
                        .as_u64()
                        .unwrap_or(0);
                }

                let name = resp
                    .get_value_by_path("data.0.attributes.name")
                    .unwrap()
                    .as_str()
                    .unwrap_or("N/A")
                    .to_string();
                media.name = name.clone();

                description = format!(
                    "Listen to {} by {} on Cider",
                    name,
                    resp.get_value_by_path("data.0.attributes.artistName")
                        .unwrap()
                        .as_str()
                        .unwrap_or("N/A")
                );
                duration = Duration::from_millis(total_duration);
            } else if url.contains("playlist") {
                let id = parsed_url.path_segments().unwrap().last().unwrap();
                let Ok(resp) = self
                    .api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/playlists/{}", storefront, id),
                    )
                    .await else {
                        eprintln!("failed to request album {id} from the apple music api");
                        return
                    };

                media.sid = id.to_string();
                media.media_type = MediaType::Playlist;

                let mut total_duration: u64 = 0;

                for i in 0..resp
                    .get_vec_len_by_path("data.0.relationships.tracks.data")
                    .unwrap()
                {
                    total_duration += resp
                        .get_value_by_path(&format!(
                            "data.0.relationships.tracks.data.{i}.attributes.durationInMillis"
                        ))
                        .unwrap()
                        .as_u64()
                        .unwrap_or(0);
                }

                let name = resp
                    .get_value_by_path("data.0.attributes.name")
                    .unwrap()
                    .as_str()
                    .unwrap_or("N/A")
                    .to_string();
                media.name = name.clone();

                description = format!(
                    "Listen to {} by {} on Cider",
                    name,
                    resp.get_value_by_path("data.0.attributes.curatorName")
                        .unwrap()
                        .as_str()
                        .unwrap_or("N/A")
                );
                duration = Duration::from_millis(total_duration);
            } else if url.contains("music-video") {
                let id = parsed_url.path_segments().unwrap().last().unwrap();
                let Ok(resp) = self
                    .api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/music-video/{}", storefront, id),
                    )
                    .await else {
                        eprintln!("failed to request album {id} from the apple music api");
                        return
                    };

                media.sid = id.to_string();
                media.media_type = MediaType::MusicVideo;

                let name = resp
                    .get_value_by_path("data.0.attributes.name")
                    .unwrap()
                    .as_str()
                    .unwrap_or("N/A")
                    .to_string();
                media.name = name.clone();

                description = format!(
                    "Listen to {} by {} on Cider",
                    name,
                    resp.get_value_by_path("data.0.attributes.artistName")
                        .unwrap()
                        .as_str()
                        .unwrap_or("N/A")
                );
                duration = Duration::from_millis(
                    resp.get_value_by_path("data.0.attributes.durationInMillis")
                        .unwrap()
                        .as_u64()
                        .unwrap_or(0),
                );
            } else if url.contains("artist") {
                let id = parsed_url.path_segments().unwrap().last().unwrap();
                let Ok(resp) = self
                    .api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/artists/{}", storefront, id),
                    )
                    .await else {
                        eprintln!("failed to request album {id} from the apple music api");
                        return
                    };

                media.sid = id.to_string();
                media.media_type = MediaType::Artist;

                let name = resp
                    .get_value_by_path("data.0.attributes.name")
                    .unwrap()
                    .as_str()
                    .unwrap_or("N/A")
                    .to_string();
                media.name = name.clone();
                description = format!("Listen to {} on Cider", name);
            } else {
                return;
            }

            // So if we create an embed each time, it takes 3 entire seconds per the call of the next three lines
            // , we need something better, like creating a wenbook the first time, then re-using.
            // let mut webhook = new_message.channel_id.create_webhook(&ctx.http, "temp-cidar").await.unwrap();
            // webhook.edit_avatar(&ctx.http, &*new_message.author.avatar_url().unwrap()).await.unwrap();
            // webhook.edit_name(&ctx.http, &new_message.author.name).await.unwrap();

            // Speed up from 4 seconds to just 0.70 ish

            // let mut webhook = match new_message.channel_id.webhooks(&ctx.http).await {
            //     Ok(hooks) => {
            //         let mut iterator = hooks.iter();
            //         // tomfoolery
            //         if let Some(webhook) = iterator.find(|&hook| {
            //             //println!("bot: {}, http: {}", &hook.user.as_ref().unwrap().id.0, &ctx.cache.current_user().id.0);
            //             &hook.user.as_ref().unwrap().id.0 == &ctx.cache.current_user().id.0
            //         }) {
            //             webhook.clone()
            //         } else {
            //             new_message.channel_id.create_webhook(&ctx.http, "cidar-webhook").await.unwrap()
            //         }
            //     },
            //     Err(_) => {
            //         // We actually dont have any webhooks in the channel, so add cidars one.
            //         new_message.channel_id.create_webhook(&ctx.http, "cidar-webhook").await.unwrap()
            //     }
            // };

            // webhook.edit_avatar(&ctx.http, &*new_message.author.avatar_url().unwrap()).await.unwrap();
            // webhook.edit_name(&ctx.http, &new_message.author.name).await.unwrap();

            let modded = url.replace("https://", "");

            let play_link = format!("https://cider.sh/p?{}", modded);
            let view_link = format!("https://cider.sh/o?{}", modded);

            new_message
                .channel_id
                .send_message(&ctx.http, |m| {
                    m.embed(|e| {
                        e.title(
                            resp.get_value_by_path("data.0.attributes.name")
                                .unwrap()
                                .as_str()
                                .unwrap_or("N/A"),
                        )
                        .url(
                            resp.get_value_by_path("data.0.attributes.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ"),
                        )
                        .thumbnail(util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        ))
                        .description(&description)
                        .footer(|f| {
                            f.text(format!(
                                "Shared by {} | {} • {}",
                                new_message.author.name,
                                util::milli_to_hhmmss(&duration),
                                resp.get_value_by_path("data.0.attributes.releaseDate")
                                    .unwrap_or(Value::String("".to_string()))
                                    .as_str()
                                    .unwrap()
                            ))
                        })
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
                .unwrap();

            // let embed = Embed::fake(|e| {
            //     e.title(resp.get_value_by_path("data.0.attributes.name").unwrap().as_str().unwrap_or("N/A"))
            //         .url(resp.get_value_by_path("data.0.attributes.url").unwrap().as_str().unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ"))
            //         .thumbnail(wh(resp.get_value_by_path("data.0.attributes.artwork.url").unwrap().as_str().unwrap(), 512, 512))
            //         .description(&description)
            //         .footer(|f| {
            //             f.text(format!("Shared by {} | {} • {}", new_message.author.name, duration.milli_to_hhmmss(), resp.get_value_by_path("data.0.attributes.releaseDate").unwrap_or(Value::String("".to_string())).as_str().unwrap()))
            //         })
            //         .timestamp(Timestamp::now())
            // });

            // webhook.execute(&ctx.http, false, |m| {
            //     m.content(content).embeds(vec![embed.clone()]).components(|c| {
            //         c.create_action_row(|r| {
            //             r.create_button(|b| {
            //                 b.label("Play in Cider")
            //                 .style(ButtonStyle::Link)
            //                 .url(play_link)
            //             })
            //             .create_button(|b| {
            //                 b.label("View in Cider")
            //                 .style(ButtonStyle::Link)
            //                 .url(view_link)
            //             })
            //         })
            //     })
            // }).await.unwrap();

            // webhook.edit_name(&ctx.http, "cidar-webhook").await.unwrap();
            // webhook.delete_avatar(&ctx.http).await.unwrap();

            // match new_message.delete(&ctx.http).await {
            //     Ok(_) => (),
            //     Err(_) => {
            //         println!("failed to delete message");
            //         return
            //     },
            // };

            new_message.suppress_embeds(&ctx.http).await.unwrap();

            // Update the conversions
            util::increment_conversion().await;
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    total_conversions: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct User {
    id: Option<Thing>,
    username: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
enum MediaType {
    #[default]
    Song,
    Album,
    Playlist,
    MusicVideo,
    Artist,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Media {
    #[serde(skip_serializing)]
    id: Option<Thing>,
    name: String,
    media_type: MediaType,

    #[serde(skip_serializing, skip_deserializing)]
    sid: String,
}

static DB: Surreal<Client> = Surreal::init();

#[tokio::main]
async fn main() {
    println!("Cidar launching");
    let token = std::env::var("TOKEN").expect("Please set the TOKEN env variable");
    let database_ip = std::env::var("DB_IP").expect("Please set the DB_IP env variable");
    let database_password = std::env::var("DB_PASS").expect("Please set the DB_PASS env variable");

    println!("Connecting to database");
    DB.connect::<Ws>(database_ip)
        .await
        .expect("Unable to connect to database");
    DB.signin(Root {
        username: "root",
        password: &database_password,
    })
    .await
    .unwrap();

    DB.use_ns("cider").use_db("cidar").await.unwrap();

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
    };

    let mut client = serenity::Client::builder(token, intents)
        .event_handler(handler)
        .framework(StandardFramework::new())
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
