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
use surrealdb::engine::remote::ws::Client;
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

#[derive(Debug, Default)]
struct EmbedInformation {
    title: String,
    description: String,
    footer: String,
    artwork: String,
    url: String,
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
                .await;

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
        const DEBUG_CHANNEL: u64 = 1133927653074796555;

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
            let storefront = match storefront.get(1) {
                Some(sf) => sf,
                None => {
                    return;
                }
            };

            // Create a place to store embed information for all of the follwing cases.
            let mut information = EmbedInformation::default();

            // Determine what type of media it is.
            if let Some(media) = MediaType::determine(&url, &query) {
                println!("Converting media type {:?}", &media);
                match media {
                    MediaType::Song => {
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

                        // return useless values instead of panicking
                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description = format!(
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

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!(
                            "Shared by {} | {} • {}",
                            new_message.author.name,
                            util::milli_to_hhmmss(&Duration::from_millis(
                                resp.get_value_by_path("data.0.attributes.durationInMillis")
                                    .unwrap()
                                    .as_u64()
                                    .unwrap_or(0),
                            )),
                            resp.get_value_by_path("data.0.attributes.releaseDate")
                                .unwrap_or(Value::String("".to_string()))
                                .as_str()
                                .unwrap()
                        )
                    }
                    MediaType::Album => {
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

                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description = format!(
                            "Listen to {} by {} on Cider",
                            &information.title,
                            resp.get_value_by_path("data.0.attributes.artistName")
                                .unwrap()
                                .as_str()
                                .unwrap_or("N/A")
                        );

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!(
                            "Shared by {} | {} • {}",
                            new_message.author.name,
                            util::milli_to_hhmmss(&Duration::from_millis(total_duration)),
                            resp.get_value_by_path("data.0.attributes.releaseDate")
                                .unwrap_or(Value::String("".to_string()))
                                .as_str()
                                .unwrap()
                        )
                    }
                    MediaType::Station => {
                        let id = parsed_url.path_segments().unwrap().last().unwrap();
                        let Ok(resp) = self
                            .api
                            .request_endpoint(
                            Method::GET,
                            &format!("v1/catalog/{}/stations/{}", storefront, id),
                        )
                        .await else {
                            eprintln!("failed to request playlists {id} from the apple music api");
                            return
                        };

                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description =
                            format!("Tune into {} on Cider", &information.title);

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!("Shared by {}", new_message.author.name)
                    }
                    MediaType::Playlist => {
                        let id = parsed_url.path_segments().unwrap().last().unwrap();
                        let Ok(resp) = self
                            .api
                            .request_endpoint(
                            Method::GET,
                            &format!("v1/catalog/{}/playlists/{}", storefront, id),
                        )
                        .await else {
                            eprintln!("failed to request playlists {id} from the apple music api");
                            return
                        };

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

                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description = format!(
                            "Listen to {} by {} on Cider",
                            &information.title,
                            resp.get_value_by_path("data.0.attributes.curatorName")
                                .unwrap()
                                .as_str()
                                .unwrap_or("N/A")
                        );

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!(
                            "Shared by {} | {}",
                            new_message.author.name,
                            util::milli_to_hhmmss(&Duration::from_millis(total_duration)),
                        )
                    }
                    MediaType::MusicVideo => {
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

                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description = format!(
                            "Listen to {} by {} on Cider",
                            &information.title,
                            resp.get_value_by_path("data.0.attributes.artistName")
                                .unwrap()
                                .as_str()
                                .unwrap_or("N/A")
                        );

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!(
                            "Shared by {} | {} • {}",
                            new_message.author.name,
                            util::milli_to_hhmmss(&Duration::from_millis(
                                resp.get_value_by_path("data.0.attributes.durationInMillis")
                                    .unwrap()
                                    .as_u64()
                                    .unwrap_or(0),
                            )),
                            resp.get_value_by_path("data.0.attributes.releaseDate")
                                .unwrap_or(Value::String("".to_string()))
                                .as_str()
                                .unwrap()
                        )
                    }
                    MediaType::Artist => {
                        let id = parsed_url.path_segments().unwrap().last().unwrap();
                        let Ok(resp) = self
                            .api
                            .request_endpoint(
                                Method::GET,
                                &format!("v1/catalog/{}/artists/{}", storefront, id),
                            )
                            .await else {
                                eprintln!("failed to request artist {id} from the apple music api");
                                return
                            };

                        information.title = resp
                            .get_value_by_path("data.0.attributes.name")
                            .unwrap()
                            .as_str()
                            .unwrap_or("N/A")
                            .to_string();

                        information.url = resp
                            .get_value_by_path("data.0.attributes.url")
                            .unwrap()
                            .as_str()
                            .unwrap_or("https://www.youtube.com/watch?v=dQw4w9WgXcQ")
                            .to_string();

                        information.description =
                            format!("Listen to {} on Cider", &information.title);

                        information.artwork = util::wh(
                            resp.get_value_by_path("data.0.attributes.artwork.url")
                                .unwrap()
                                .as_str()
                                .unwrap_or(""),
                            512,
                            512,
                        );

                        information.footer = format!("Shared by {}", new_message.author.name)
                    }
                }
            } else {
                // We dont support whatever they are trying to convert, so bail out.
                return;
            }

            let modded = url.replace("https://", "");

            let play_link = format!("https://cider.sh/p?{}", modded);
            let view_link = format!("https://cider.sh/o?{}", modded);

            new_message
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
                .unwrap();

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
enum MediaType {
    #[default]
    Song,
    Album,
    Playlist,
    MusicVideo,
    Station,
    Artist,
}

impl MediaType {
    fn determine(url: &str, query: &HashMap<String, String>) -> Option<MediaType> {
        // https://music.apple.com/us/artist/dax/1368102340
        // We need to get the 1st index of this to properly match the strings.
        let url = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return None,
        };

        let segments = url.path_segments().unwrap().collect::<Vec<&str>>();

        let media_type = match segments.get(1) {
            Some(t) => t,
            None => return None,
        };

        // Handle the album edge case where it MAY have a song ID.
        if let Some(_) = query.get("i") {
            Some(MediaType::Song)
        } else {
            // Do the regular paring using the url identifiers
            match *media_type {
                "song" => Some(MediaType::Song),
                "album" => Some(MediaType::Album),
                "artist" => Some(MediaType::Artist),
                "music-video" => Some(MediaType::MusicVideo),
                "playlist" => Some(MediaType::Playlist),
                "station" => Some(MediaType::Station),
                _ => {
                    println!("Unknown media type {}", media_type);
                    println!("info:");
                    println!("\turl: {}", &url);
                    None
                }
            }
        }
    }
}

static DB: Surreal<Client> = Surreal::init();

#[tokio::main]
async fn main() {
    println!("Cidar launching");

    let token = std::env::var("TOKEN").expect("Please set the TOKEN env variable");

    println!("Starting crash governer");

    let _guard = sentry::init(("https://15cf6882a0fd0152775f80dbbf4b1c4e@o4504730117865472.ingest.sentry.io/4505693108371456", sentry::ClientOptions {
        release: sentry::release_name!(),
        ..Default::default()
    }));

    util::connect_to_db().await;

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
