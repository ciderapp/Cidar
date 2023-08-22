use std::{net::TcpStream, sync::Arc, time::Duration};

use log::*;
use surrealdb::{engine::remote::ws::Ws, opt::auth::Root, Error};
use tokio::sync::RwLock;

use crate::{Store, DATABASE_IP, DATABASE_PASSWORD};

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

/// Conversion cache type for when we cannot send events to the database.
pub type Cache = Arc<RwLock<Option<Store>>>;

pub async fn increment_conversion(cache: Cache) {
    if cache.read().await.is_none() {
        // Create the cache
        info!("Creating cache for total conversions");
        if !db_online().await {
            warn!("Unable to create cache due to lack of server connectivity");
            return;
        }
    }

    if db_online().await {
        let read_store: Store = match crate::DB.select(("stats", "conversions")).await {
            Ok(s) => s,
            Err(_) => {
                info!("Creating counter in DB");
                create_conversion_counter().await
            }
        };

        // Determine which one to use.
        if cache.read().await.is_some()
            && read_store.total_conversions < cache.read().await.unwrap().total_conversions
        {
            // Write the highest value to cache

            info!(
                "Overriding db value with that in cache {} -> {}",
                read_store.total_conversions,
                cache.read().await.unwrap().total_conversions
            );

            let read_store = cache.read().await.unwrap();
            *cache.write().await = Some(read_store);
        } else {
            // Always write to cache, because that is what we will use for calculations
            *cache.write().await = Some(read_store);
        }
    }

    // Read the cache again, increment, then save back to the cache
    let mut read_store = cache.read().await.unwrap();
    read_store.total_conversions += 1;
    *cache.write().await = Some(read_store);

    let conversion = cache.read().await.unwrap().total_conversions;
    trace!("total conversions = {}", conversion);
    drop(cache);

    let s: Result<Store, Error> = crate::DB
        .update(("stats", "conversions"))
        .content(read_store)
        .await;

    if s.is_err() {
        error!("Unable to update conversions in the database, storing locally in cache.");
    }
}

async fn create_conversion_counter() -> Store {
    let Ok(s) = crate::DB
        .create(("stats", "conversions"))
        .content(Store::default())
        .await
    else {
        warn!("Failed to create conversion source. most likely made already");
        return Store::default();
    };
    s
}

pub fn split_authors(authors: &str) -> String {
    authors.split(':').collect::<Vec<&str>>().join(", ")
}

pub async fn connect_to_db() -> Result<(), Error> {
    crate::DB.connect::<Ws>(DATABASE_IP.as_str()).await?;
    crate::DB
        .signin(Root {
            username: "root",
            password: &DATABASE_PASSWORD,
        })
        .await?;

    crate::DB.use_ns("cider").use_db("cidar").await?;
    Ok(())
}

pub async fn db_online() -> bool {
    TcpStream::connect_timeout(
        &DATABASE_IP.as_str().parse().unwrap(),
        Duration::from_millis(500),
    ).is_ok()
}
