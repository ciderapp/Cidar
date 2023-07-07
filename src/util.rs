use std::time::Duration;

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
        Err(_) => {
            let s: Store = crate::DB
                .create(("stats", "conversions"))
                .content(Store::default())
                .await
                .unwrap();
            s
        }
    };
    read_store.total_conversions += 1;

    let _: Store = crate::DB
        .update(("stats", "conversions"))
        .content(read_store)
        .await
        .unwrap();
}
