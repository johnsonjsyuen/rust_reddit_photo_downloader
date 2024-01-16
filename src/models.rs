use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListingDetail {
    pub(crate) title: String,
    pub(crate) id: String,
    pub(crate) url: String,
    pub(crate) is_video: bool,
    pub(crate) domain: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Listing {
    pub(crate) data: ListingDetail,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListingData {
    pub(crate) children: Vec<Listing>,
    pub(crate) after: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ListingResponse {
    pub(crate) kind: String,
    pub(crate) data: ListingData,
}
