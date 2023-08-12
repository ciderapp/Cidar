use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Station {
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
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attributes {
    pub artwork: Artwork,
    pub media_kind: String,
    pub is_live: bool,
    pub name: String,
    pub play_params: PlayParams,
    pub supported_drms: Option<Vec<String>>,
    pub editorial_notes: Option<EditorialNotes>,
    pub url: String,
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
    pub format: String,
    pub station_hash: String,
    pub has_drm: bool,
    pub media_type: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorialNotes {
    pub name: Option<String>,
    pub short: Option<String>,
    pub tagline: Option<String>,
}