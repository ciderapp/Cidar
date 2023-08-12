use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Playlist {
    pub data: Vec<Daum>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: Option<String>,
    pub attributes: Attributes,
    pub relationships: Relationships,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub artwork: Artwork,
    pub is_chart: bool,
    pub url: String,
    pub last_modified_date: String,
    pub name: String,
    pub playlist_type: String,
    pub curator_name: String,
    pub play_params: PlayParams,
    pub description: Description,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artwork {
    pub width: i64,
    pub height: i64,
    pub url: String,
    pub bg_color: Option<String>,
    pub text_color1: Option<String>,
    pub text_color2: Option<String>,
    pub text_color3: Option<String>,
    pub text_color4: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayParams {
    pub id: String,
    pub kind: String,
    pub version_hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Description {
    pub standard: Option<String>,
    pub short: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Relationships {
    pub tracks: Tracks,
    pub curator: Curator,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tracks {
    pub href: Option<String>,
    pub data: Vec<Daum2>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum2 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: Option<String>,
    pub attributes: Attributes2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes2 {
    pub previews: Vec<Preview>,
    pub artwork: Artwork2,
    pub artist_name: String,
    pub url: String,
    pub disc_number: u64,
    pub genre_names: Vec<String>,
    pub duration_in_millis: u64,
    pub release_date: String,
    pub is_apple_digital_master: bool,
    pub name: String,
    pub isrc: String,
    pub has_lyrics: bool,
    pub album_name: String,
    pub play_params: PlayParams2,
    pub track_number: u64,
    pub composer_name: Option<String>,
    pub content_rating: Option<String>,
    pub editorial_notes: Option<EditorialNotes>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    pub url: String,
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
pub struct EditorialNotes {
    pub short: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Curator {
    pub href: Option<String>,
    pub data: Vec<Daum3>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Daum3 {
    pub id: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub href: Option<String>,
}