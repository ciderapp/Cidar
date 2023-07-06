use std::error::Error;
use std::fmt;

use regex::Regex;
use serde_json::Value;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use serenity::{
    builder::CreateApplicationCommand, model::prelude::application_command::CommandDataOptionValue,
};

use crate::{AppleMusicApi, ValuePath, increment_conversion};

#[derive(Debug)]
struct ConvertError {
    message: String,
}

impl ConvertError {
    fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Error for ConvertError {}

impl fmt::Display for ConvertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Convert Error: {}", self.message)
    }
}

pub async fn run(
    _options: &[CommandDataOption],
    token: &AppleMusicApi,
    regex: &Regex,
) -> Result<String, Box<dyn Error>> {
    let opt = match _options.get(0) {
        Some(o) => o.resolved.as_ref(),
        None => return Err(Box::new(ConvertError::new("Did not input link"))),
    };

    if let CommandDataOptionValue::String(str) = opt.unwrap() {
        if !regex.is_match(&str) {
            return Err(Box::new(ConvertError::new("Content is not a link")));
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
                    increment_conversion().await;
                    url.as_str().unwrap().to_string()
                },
                None => {
                    return Err(Box::new(ConvertError::new(
                        "Unable to convert to apple music link",
                    )))
                }
            },
        )
    } else {
        Err(Box::new(ConvertError::new("Option was **not** a string")))
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
