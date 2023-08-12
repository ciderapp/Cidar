use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Album {
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
    pub copyright: String,
    pub genre_names: Vec<String>,
    pub release_date: String,
    pub upc: String,
    pub is_mastered_for_itunes: bool,
    pub artwork: Artwork,
    pub play_params: PlayParams,
    pub url: String,
    pub record_label: String,
    pub is_compilation: bool,
    pub track_count: i64,
    pub is_single: bool,
    pub name: String,
    pub content_rating: String,
    pub artist_name: String,
    pub editorial_notes: Option<EditorialNotes>,
    pub is_complete: bool,
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
pub struct EditorialNotes {
    pub standard: String,
    pub short: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationships {
    pub artists: Artists,
    pub tracks: Tracks,
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
pub struct Tracks {
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
    pub attributes: Attributes2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes2 {
    pub album_name: String,
    pub genre_names: Vec<String>,
    pub track_number: u64,
    pub release_date: String,
    pub duration_in_millis: u64,
    pub isrc: String,
    pub artwork: Artwork2,
    pub composer_name: String,
    pub play_params: PlayParams2,
    pub url: String,
    pub disc_number: i64,
    pub is_apple_digital_master: bool,
    pub has_lyrics: bool,
    pub name: String,
    pub previews: Vec<Preview>,
    pub artist_name: String,
    pub content_rating: Option<String>,
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
pub struct PlayParams2 {
    pub id: String,
    pub kind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub url: String,
}