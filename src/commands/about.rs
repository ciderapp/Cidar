use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;
use crate::util::split_authors;

trait NaIfNone {
    fn na(&self) -> String;
}

impl NaIfNone for Option<&str> {
    fn na(&self) -> String {
        self.unwrap_or("N/A").to_string()
    }
}

pub fn run(_options: &[CommandDataOption]) -> String {
    format!(
        "Version: {}
Author(s): {}
Build time: {}
Commit hash: {}
Rust version: {}",
        option_env!("CARGO_PKG_VERSION").na(),
        split_authors(&option_env!("CARGO_PKG_AUTHORS").na()),
        option_env!("VERGEN_BUILD_TIMESTAMP").na(),
        option_env!("VERGEN_GIT_SHA").na(),
        option_env!("VERGEN_RUSTC_SEMVER").na()
    )
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("about")
        .description("Get information about Cidar")
}
