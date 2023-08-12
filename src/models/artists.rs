use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Artist {
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
    pub genre_names: Vec<String>,
    pub name: String,
    pub artwork: Artwork,
    pub url: String,
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
pub struct Relationships {
    pub albums: Albums,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Albums {
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
