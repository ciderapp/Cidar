use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongLink {
    pub entity_unique_id: String,
    pub user_country: String,
    pub page_url: String,
    pub links_by_platform: LinksByPlatform,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinksByPlatform {
    pub apple_music: AppleMusic,
    pub spotify: Spotify,
    pub youtube: Youtube,
    pub youtube_music: YoutubeMusic,
    pub google: Google,
    pub pandora: Pandora,
    pub deezer: Deezer,
    pub amazon_music: AmazonMusic,
    pub tidal: Tidal,
    pub napster: Napster,
    pub yandex: Yandex,
    pub itunes: Itunes,
    pub google_store: GoogleStore,
    pub amazon_store: AmazonStore,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppleMusic {
    pub url: String,
    pub native_app_uri_mobile: String,
    pub native_app_uri_desktop: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spotify {
    pub url: String,
    pub native_app_uri_desktop: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Youtube {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YoutubeMusic {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Google {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pandora {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deezer {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmazonMusic {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tidal {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Napster {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Yandex {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Itunes {
    pub url: String,
    pub native_app_uri_mobile: String,
    pub native_app_uri_desktop: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoogleStore {
    pub url: String,
    pub entity_unique_id: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmazonStore {
    pub url: String,
    pub entity_unique_id: String,
}
