use std::time::Duration;

use surrealdb::{engine::remote::ws::Ws, opt::auth::Root};

use crate::Store;

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

pub async fn increment_conversion() {
    let mut read_store: Store = match crate::DB.select(("stats", "conversions")).await {
        Ok(s) => s,
        Err(_) => create_conversion_counter().await,
    };

    read_store.total_conversions += 1;

    crate::DB
        .update(("stats", "conversions"))
        .content(read_store)
        .await
        .unwrap_or_else(|_| eprintln!("Failed to update conversions"));
}

async fn create_conversion_counter() -> Store {
    let Ok(s) = crate::DB
        .create(("stats", "conversions"))
        .content(Store::default())
        .await else {
            panic!("Failed to create conversions store")
        };

    s
}

pub fn split_authors(authors: &str) -> String {
    authors.split(':').collect::<Vec<&str>>().join(", ")
}

pub async fn connect_to_db() {
    let database_ip = std::env::var("DB_IP").expect("Please set the DB_IP env variable");
    let database_password = std::env::var("DB_PASS").expect("Please set the DB_PASS env variable");

    println!("Connecting to SurrealDB @ {}", database_ip);

    crate::DB.connect::<Ws>(database_ip)
        .await
        .expect("Unable to connect to database");
    crate::DB.signin(Root {
        username: "root",
        password: &database_password,
    })
    .await
    .unwrap();

    crate::DB.use_ns("cider").use_db("cidar").await.unwrap();
}