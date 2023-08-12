use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicVideo {
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub previews: Vec<Preview>,
    pub artwork: Artwork2,
    pub artist_name: String,
    pub url: String,
    pub genre_names: Vec<String>,
    #[serde(rename = "has4K")]
    pub has4k: bool,
    pub duration_in_millis: u64,
    pub release_date: String,
    pub name: String,
    pub isrc: String,
    pub play_params: PlayParams,
    #[serde(rename = "hasHDR")]
    pub has_hdr: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub url: String,
    pub hls_url: String,
    pub artwork: Artwork,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub width: i64,
    pub height: i64,
    pub url: String,
    pub bg_color: String,
    pub text_color1: String,
    pub text_color2: String,
    pub text_color3: String,
    pub text_color4: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork2 {
    pub width: i64,
    pub height: i64,
    pub url: String,
    pub bg_color: String,
    pub text_color1: String,
    pub text_color2: String,
    pub text_color3: String,
    pub text_color4: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayParams {
    pub id: String,
    pub kind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationships {
    pub artists: Artists,
    pub albums: Albums,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artists {
    pub href: String,
    pub data: Vec<Daum2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum2 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Albums {
    pub href: String,
    pub data: Vec<Value>,
}