use crate::util::split_authors;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::CommandDataOption;

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
Commit hash: [{hash}](https://github.com/ciderapp/Cidar/commit/{hash})
Rust version: {}",
        option_env!("CARGO_PKG_VERSION").na(),
        split_authors(&option_env!("CARGO_PKG_AUTHORS").na()),
        option_env!("VERGEN_BUILD_TIMESTAMP").na(),
        option_env!("VERGEN_RUSTC_SEMVER").na(),
        hash=option_env!("VERGEN_GIT_SHA").na()
    )
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("about")
        .description("Get information about Cidar")
}
