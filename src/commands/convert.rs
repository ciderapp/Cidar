use thiserror::Error;

use regex::Regex;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::{
    builder::CreateApplicationCommand, model::prelude::application_command::CommandDataOptionValue,
};

use crate::models;
use crate::api::AppleMusicApi;

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("did not input link")]
    InvalidInput,
    #[error("content is not a link")]
    InvalidContent,
    #[error("could not convert to apple music link")]
    InvalidOption,
    #[error("request failed")]
    RequestError(String),
}

impl From<reqwest::Error> for ConvertError {
    fn from(value: reqwest::Error) -> Self {
        ConvertError::RequestError(value.to_string())
    }
}

pub async fn run(
    _options: &[CommandDataOption],
    token: &AppleMusicApi,
    regex: &Regex,
) -> Result<String, ConvertError> {
    let opt = match _options.get(0) {
        Some(o) => o.resolved.as_ref(),
        None => return Err(ConvertError::InvalidInput),
    };

    if let CommandDataOptionValue::String(str) = opt.unwrap() {
        if !regex.is_match(str) {
            return Err(ConvertError::InvalidContent);
        }

        let response: models::songlink::SongLink = token
            .client
            .read()
            .await
            .get(format!(
                "https://api.song.link/v1-alpha.1/links?url={}",
                str
            ))
            .send()
            .await?
            .json::<models::songlink::SongLink>()
            .await?;

        Ok(response.links_by_platform.apple_music.url)
    } else {
        Err(ConvertError::InvalidOption)
    }
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("convert")
        .description("Converts any link into an apple equivilent")
        .create_option(|option| {
            option
                .name("link")
                .description("Media link")
                .kind(CommandOptionType::String)
                .required(true)
        })
}
