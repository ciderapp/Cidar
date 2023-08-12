use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
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
    pub album_name: String,
    pub genre_names: Vec<String>,
    pub track_number: u64,
    pub duration_in_millis: u64,
    pub release_date: String,
    pub isrc: String,
    pub artwork: Artwork,
    pub composer_name: String,
    pub play_params: PlayParams,
    pub url: String,
    pub disc_number: i64,
    pub has_lyrics: bool,
    pub is_apple_digital_master: bool,
    pub name: String,
    pub previews: Vec<Preview>,
    pub artist_name: String,
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
pub struct PlayParams {
    pub id: String,
    pub kind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub url: String,
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
    pub data: Vec<Daum3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum3 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: String,
}