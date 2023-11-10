use once_cell::sync::Lazy;
use std::{path::PathBuf, str::FromStr, time::Duration};
use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncReadExt, AsyncWriteExt},
};

use log::*;
use serde::{Deserialize, Serialize};

use crate::Stats;

pub fn milli_to_hhmmss(duration: &Duration) -> String {
    let millis = duration.as_millis();

    let seconds = millis / 1000;
    let ss = seconds % 60;
    let mm = (seconds / 60) % 60;
    let hh = (seconds / (60 * 60)) % 24;

    if hh == 0 && mm != 0 {
        format!("{:02}:{:02}", mm, ss)
    } else if hh == 0 && mm == 0 {
        format!("{:02}", ss)
    } else {
        format!("{}:{:02}:{:02}", hh, mm, ss)
    }
}

pub fn wh(url: &str, w: u32, h: u32) -> String {
    url.replace("{w}", &format!("{}", w))
        .replace("{h}", &format!("{}", h))
}

pub static CFG_PATH: Lazy<PathBuf> = Lazy::new(|| {
    PathBuf::from_str(
        &std::env::var("CFG_PATH").unwrap_or(
            std::env::current_dir()
                .expect("Could not fall back to current working directory")
                .to_string_lossy()
                .to_string(),
        ),
    )
    .expect("Not a valid path config")
});

pub async fn increment_conversion() -> Result<(), ()> {
    let mut opts = OpenOptions::new();
    opts.read(true);
    opts.write(true);
    opts.create(true);

    let mut file = opts
        .clone()
        .open(CFG_PATH.join("stats.json"))
        .await
        .expect("Unable to open stats.json");

    let mut read_stats: Stats = match read_json(&mut file).await {
        Ok(s) => s,
        Err(_) => create_conversion_counter().await,
    };

    info!(
        "incrementing conversions; {} -> {}",
        read_stats.total_conversions,
        read_stats.total_conversions + 1
    );
    read_stats.total_conversions += 1;

    opts.truncate(true);
    let mut file = opts
        .clone()
        .open(CFG_PATH.join("stats.json"))
        .await
        .expect("Unable to open stats.json");

    write_json(&read_stats, &mut file).await;

    Ok(())
}

pub async fn create_conversion_counter() -> Stats {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(CFG_PATH.join("stats.json"))
        .await
        .expect("Unable to open stats.json");

    let stats = Stats::default();
    if let Err(e) = file
        .write_all(serde_json::to_string_pretty(&stats).unwrap().as_bytes())
        .await
    {
        warn!("Unable to write stats.json: {}", e.to_string())
    }

    stats
}

pub fn split_authors(authors: &str) -> String {
    authors.split(':').collect::<Vec<&str>>().join(", ")
}

pub async fn write_json<T: Serialize + for<'a> Deserialize<'a>>(json: &T, file: &mut File) {
    let _ = file.write(&serde_json::to_vec_pretty(json).unwrap()).await;
}

pub async fn read_json<T: Serialize + for<'a> Deserialize<'a>>(
    file: &mut File,
) -> Result<T, serde_json::Error> {
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).await.unwrap();

    serde_json::from_str::<T>(&buffer)
}
