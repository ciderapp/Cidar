use std::{collections::HashMap, time::Duration};

use log::*;
use reqwest::{Method, Url};
use serde::{Deserialize, Serialize};

use crate::{api::AppleMusicApi, util, vpath::ValuePath};

#[derive(Debug, Default)]
pub struct EmbedInformation {
    pub title: String,
    pub description: String,
    pub footer: String,
    pub artwork: String,
    pub url: String,
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
    fn determine(url: &Url, query: &HashMap<String, String>) -> Option<MediaType> {
        let segments = url.path_segments()?.collect::<Vec<&str>>();

        let media_type = segments.get(1)?;

        // Handle the album edge case where it MAY have a song ID.
        if query.get("i").is_some() {
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
                    warn!("Unknown media type {}", media_type);
                    info!("\turl: {}", &url);
                    None
                }
            }
        }
    }
}

pub async fn get_information(
    api: &AppleMusicApi,
    url: &Url,
    storefront: &str,
    query: &HashMap<String, String>,
    message: &serenity::model::prelude::Message,
) -> Option<EmbedInformation> {
    // Create a place to store embed information for all of the follwing cases.
    let mut information = EmbedInformation::default();

    // Determine what type of media it is.
    if let Some(media) = MediaType::determine(url, query) {
        info!("Converting media type {:?}", &media);
        match media {
            MediaType::Song => {
                let id = match query.get("i") {
                    Some(i) => i,
                    None => url.path_segments()?.last()?,
                };

                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/songs/{}", storefront, &id),
                    )
                    .await
                else {
                    warn!("failed to request song {id} from the apple music api");
                    return None;
                };

                // return useless values instead of panicking
                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!(
                    "Listen to {} by {} on Cider",
                    resp.get_value_by_path("data.0.attributes.albumName")?
                        .as_str()?,
                    resp.get_value_by_path("data.0.attributes.artistName")?
                        .as_str()?
                );

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")
                        .unwrap()
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!(
                    "Shared by {} | {} • {}",
                    message.author.name,
                    util::milli_to_hhmmss(&Duration::from_millis(
                        resp.get_value_by_path("data.0.attributes.durationInMillis")?
                            .as_u64()
                            .unwrap_or(0),
                    )),
                    resp.get_value_by_path("data.0.attributes.releaseDate")?
                        .as_str()?
                )
            }
            MediaType::Album => {
                let id = url.path_segments()?.last()?;
                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/albums/{}", storefront, id),
                    )
                    .await
                else {
                    warn!("failed to request album {id} from the apple music api");
                    return None;
                };

                let mut total_duration: u64 = 0;

                for i in 0..resp.get_vec_len_by_path("data.0.relationships.tracks.data")? {
                    total_duration += resp
                        .get_value_by_path(&format!(
                            "data.0.relationships.tracks.data.{i}.attributes.durationInMillis"
                        ))
                        .unwrap()
                        .as_u64()
                        .unwrap_or(0);
                }

                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!(
                    "Listen to {} by {} on Cider",
                    &information.title,
                    resp.get_value_by_path("data.0.attributes.artistName")?
                        .as_str()?
                );

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")?
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!(
                    "Shared by {} | {} • {}",
                    message.author.name,
                    util::milli_to_hhmmss(&Duration::from_millis(total_duration)),
                    resp.get_value_by_path("data.0.attributes.releaseDate")?
                        .as_str()?
                )
            }
            MediaType::Station => {
                let id = url.path_segments()?.last()?;
                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/stations/{}", storefront, id),
                    )
                    .await
                else {
                    warn!("failed to request playlists {id} from the apple music api");
                    return None;
                };

                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!("Tune into {} on Cider", &information.title);

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")?
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!("Shared by {}", message.author.name)
            }
            MediaType::Playlist => {
                let id = url.path_segments()?.last()?;
                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/playlists/{}", storefront, id),
                    )
                    .await
                else {
                    warn!("failed to request playlists {id} from the apple music api");
                    return None;
                };

                let mut total_duration: u64 = 0;

                for i in 0..resp.get_vec_len_by_path("data.0.relationships.tracks.data")? {
                    total_duration += resp
                        .get_value_by_path(&format!(
                            "data.0.relationships.tracks.data.{i}.attributes.durationInMillis"
                        ))?
                        .as_u64()
                        .unwrap_or(0);
                }

                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!(
                    "Listen to {} by {} on Cider",
                    &information.title,
                    resp.get_value_by_path("data.0.attributes.curatorName")?
                        .as_str()
                        .unwrap_or("N/A")
                );

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")?
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!(
                    "Shared by {} | {}",
                    message.author.name,
                    util::milli_to_hhmmss(&Duration::from_millis(total_duration)),
                )
            }
            MediaType::MusicVideo => {
                let id = url.path_segments()?.last()?;
                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/music-video/{}", storefront, id),
                    )
                    .await
                else {
                    warn!("failed to request album {id} from the apple music api");
                    return None;
                };

                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!(
                    "Listen to {} by {} on Cider",
                    &information.title,
                    resp.get_value_by_path("data.0.attributes.artistName")?
                        .as_str()?
                );

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")?
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!(
                    "Shared by {} | {} • {}",
                    message.author.name,
                    util::milli_to_hhmmss(&Duration::from_millis(
                        resp.get_value_by_path("data.0.attributes.durationInMillis")?
                            .as_u64()
                            .unwrap_or(0),
                    )),
                    resp.get_value_by_path("data.0.attributes.releaseDate")?
                        .as_str()?
                )
            }
            MediaType::Artist => {
                let id = url.path_segments()?.last()?;
                let Ok(resp) = api
                    .request_endpoint(
                        Method::GET,
                        &format!("v1/catalog/{}/artists/{}", storefront, id),
                    )
                    .await
                else {
                    warn!("failed to request artist {id} from the apple music api");
                    return None;
                };

                information.title = resp
                    .get_value_by_path("data.0.attributes.name")?
                    .as_str()?
                    .to_string();

                information.url = resp
                    .get_value_by_path("data.0.attributes.url")?
                    .as_str()?
                    .to_string();

                information.description = format!("Listen to {} on Cider", &information.title);

                information.artwork = util::wh(
                    resp.get_value_by_path("data.0.attributes.artwork.url")?
                        .as_str()?,
                    512,
                    512,
                );

                information.footer = format!("Shared by {}", message.author.name)
            }
        }
    } else {
        // We dont support whatever they are trying to convert, so bail out.
        return None;
    }

    Some(information)
}
