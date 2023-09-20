use thiserror::Error;

use regex::Regex;
use serde_json::Value;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::{
    builder::CreateApplicationCommand, model::prelude::application_command::CommandDataOptionValue,
};

use crate::util::Cache;
use crate::{api::AppleMusicApi, util, ValuePath};

#[derive(Error, Debug)]
pub enum ConvertError {
    #[error("did not input link")]
    InvalidInput,
    #[error("content is not a link")]
    InvalidContent,
    #[error("could not convert to apple music link")]
    FailedConversion,
    #[error("option was not a string")]
    InvalidOption,
    #[error("request failed")]
    RequestError(#[from] reqwest::Error),
}

pub async fn run(
    _options: &[CommandDataOption],
    token: &AppleMusicApi,
    regex: &Regex,
    cache: &Cache,
) -> Result<String, ConvertError> {
    let opt = match _options.get(0) {
        Some(o) => o.resolved.as_ref(),
        None => return Err(ConvertError::InvalidInput),
    };

    if let CommandDataOptionValue::String(str) = opt.unwrap() {
        if !regex.is_match(str) {
            return Err(ConvertError::InvalidContent);
        }

        let response: Value = token
            .client
            .read()
            .await
            .get(format!(
                "https://api.song.link/v1-alpha.1/links?url={}",
                str
            ))
            .send()
            .await?
            .json()
            .await?;

            Ok(
                match response.get_value_by_path("linksByPlatform.appleMusic.url") {
                    Some(url) => {
                        util::increment_conversion(cache.clone()).await;
                        url.as_str().unwrap().to_string()
                    }
                    None => return Err(ConvertError::FailedConversion),
                },
            )
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
