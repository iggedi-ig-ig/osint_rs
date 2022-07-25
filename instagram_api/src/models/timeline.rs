use crate::models::{InstagramUser, Timestamp};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct InstagramTimeline {
    pub num_results: i32,
    pub feed_items: Vec<TimelineItem>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub enum TimelineItem {
    MediaOrAd {
        taken_at: Timestamp,
        pk: String,
        id: String,
        device_timestamp: Timestamp,
        media_type: i32, // TODO: this can probably be an enum

        /// sometimes a `media_or_add` has _either_ carousel items, _or_ the image versions as a field directly.
        /// This might be related to the media type, so at a later point it  might be better to make an
        /// enum variant for every media_type there is (maybe with serde_repr?)
        #[serde(flatten)]
        media_data: MediaData,
        user: InstagramUser, // TODO
        comment_count: u32,
        like_count: u32,
        caption: MediaCaption,
    },
    SuggestedUsers {
        #[serde(rename = "type")]
        suggested_type: i32,
        suggestions: Vec<UserSuggestion>,
        title: String,
        landing_site_title: String,
        id: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum MediaData {
    Carousel {
        carousel_media: Vec<CarouselItem>,
    },
    UnNested {
        #[serde(rename = "image_versions2")]
        image_versions: ImageCandidates,
    },
}

#[derive(Deserialize, Debug)]
pub struct CarouselItem {
    pub id: String,
    pub media_type: i32, // TODO: this can probably be an enum

    #[serde(rename = "image_versions2")]
    pub image_versions: ImageCandidates,
}

#[derive(Deserialize, Debug)]
pub struct ImageCandidates {
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Candidate {
    pub width: u32,
    pub height: u32,
    pub url: String,
}

#[derive(Deserialize, Debug)]
pub struct MediaCaption {
    pub pk: String,
    pub user_id: u64,
    pub text: String,
    #[serde(rename = "type")]
    pub media_type: i32, // TODO: this can probably be an enum
    pub created_at: Timestamp,
    pub created_at_utc: Timestamp,
    pub user: InstagramUser,
}

#[derive(Deserialize, Debug)]
pub struct UserSuggestion {
    pub user: InstagramUser,
    pub algorithm: String,      // TODO: this could maybe be an enum
    pub social_context: String, // TODO: this can probably be an enum
    pub caption: String,
}

#[test]
pub fn test_deserialize_timeline() {
    let test_data = include_str!("../../../test_data/timeline.json");
    let timeline: InstagramTimeline = serde_json::from_str(test_data).unwrap();
    println!("{:#?}", timeline);
}
