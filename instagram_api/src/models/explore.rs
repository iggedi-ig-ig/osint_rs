use crate::models::timeline::MediaData;
use crate::models::{InstagramIdentifier, InstagramUser, Timestamp};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct InstagramHashtag {
    pub data: ExploreData,
}

#[derive(Deserialize, Debug)]
pub struct ExploreData {
    pub id: InstagramIdentifier,
    pub name: String,
    pub media_count: u32,
    pub profile_pic_url: String,
    pub top: PostTab,
    pub recent: PostTab,
}

#[derive(Deserialize, Debug)]
pub struct PostTab {
    pub sections: Vec<PostSection>,
    pub more_available: bool,
    pub next_max_id: String,
    pub next_media_ids: Vec<InstagramIdentifier>,
}

#[derive(Deserialize, Debug)]
pub struct PostSection {
    pub layout_type: String, // TODO: this can probably be an enum
    pub layout_content: LayoutContent,
    pub feed_type: String, // TODO: this can probably be an enum
    pub explore_item_info: ExploreItemInfo,
}

#[derive(Deserialize, Debug)]
pub struct LayoutContent {
    pub medias: Vec<Media>,
}

#[derive(Deserialize, Debug)]
pub struct ExploreItemInfo {
    pub num_columns: u32,
    pub total_num_columns: u32,
    pub aspect_ratio: f32,
    pub autoplay: bool,
}

#[derive(Deserialize, Debug)]
pub struct Media {
    pub media: MediaChild,
}

#[derive(Deserialize, Debug)]
pub struct MediaChild {
    pub taken_at: Timestamp,
    pub pk: InstagramIdentifier,
    pub id: InstagramIdentifier,
    pub accessibility_caption: Option<String>,
    pub location: Option<Location>,
    pub user: InstagramUser,
    #[serde(rename = "image_versions2")]
    #[serde(flatten)]
    pub media_data: MediaData,
    pub caption: Option<PostCaption>,
    #[serde(flatten)]
    pub original_size: Option<OriginalSizeData>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Location {
    pub pk: InstagramIdentifier,
    pub short_name: String,
    pub name: String,
    pub lat: f32,
    pub lng: f32,
}

#[derive(Deserialize, Debug)]
pub struct OriginalSizeData {
    pub original_width: u32,
    pub original_height: u32,
}

#[derive(Deserialize, Debug)]
pub struct Comment {
    pub pk: InstagramIdentifier,
    pub user_id: InstagramIdentifier,
    pub text: String,
    #[serde(rename = "type")]
    pub comment_type: i32, // TODO: this can probably be an enum
    pub created_at: Timestamp,
    pub created_at_utc: Timestamp,
    pub content_type: String, // TODO: this can probably be an enum
    pub status: String,       // TODO: this can probably be an enum
    pub user: InstagramUser,
    pub media_id: InstagramIdentifier,
}

#[derive(Deserialize, Debug)]
pub struct PostCaption {
    pub pk: InstagramIdentifier,
    pub user_id: InstagramIdentifier,
    pub text: String,
    #[serde(rename = "type")]
    pub post_type: i32, // TODO: this can probably be an enum
    pub created_at: Timestamp,
    pub created_at_utc: Timestamp,
}

#[test]
pub fn test_deserialize_explore_data() {
    let test_data = include_str!("../../../test_data/explore_tag.json");
    let hashtag_data: InstagramHashtag = serde_json::from_str(test_data).unwrap();
    println!("{:#?}", hashtag_data)
}
