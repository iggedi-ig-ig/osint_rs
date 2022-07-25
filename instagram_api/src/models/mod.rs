/// A model type of any Instagram Api object has to have the prefix "Instagram"
/// example: `InstagramReelsTray`, `InstagramUser`, `InstagramTimeline`
/// Any types that are used as types within such api objects have to not
/// have any such prefixes
/// (except for if they're necessary to differentiate between other types,
/// as is the case with e.g. `SuggestedUser` ).
/// Any model type has to be in their own descriptively named file.
/// All auxiliary objects used in such types are to be placed in the same
/// file as their root object.
///
/// Objects that are used in a generic manner
/// (as is the case with e.g. `InstagramUser` or `Timestamp`)
/// are to be placed in the current file, `mod.rs`.
///
/// All fields of data structs are to be made `pub`
use serde::Deserialize;

pub mod actions;
pub mod explore;
pub mod graph_ql;
pub mod login;
pub mod reels;
pub mod timeline;

pub type Timestamp = u64;

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
/// this represents an id of an object. Can represent users, hashtags, media, etc.
/// Instagram uses numerical and string values
/// for ids interchangeably so this has to be an enum
pub enum InstagramIdentifier {
    Numeric(u64),
    Literal(String),
}

impl ToString for InstagramIdentifier {
    fn to_string(&self) -> String {
        match self {
            InstagramIdentifier::Numeric(id) => id.to_string(),
            InstagramIdentifier::Literal(id) => id.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
#[allow(clippy::large_enum_variant)]
pub struct InstagramUser {
    pub pk: InstagramIdentifier,
    #[serde(flatten)]
    pub profile_name_data: Option<ProfileNameData>,
    #[serde(flatten)]
    pub profile_pic_data: Option<ProfilePicData>,
    pub friendship_status: Option<FriendshipStatus>,
}

#[derive(Deserialize, Debug)]
pub struct ProfileNameData {
    pub username: String,
    pub full_name: String,
}

#[derive(Deserialize, Debug)]
pub struct ProfilePicData {
    pub profile_pic_url: String,
    pub profile_pic_id: String,
}

#[derive(Deserialize, Debug)]
pub struct FriendshipStatus {
    pub following: bool,
    pub outgoing_request: bool,
}

#[derive(Deserialize, Debug)]
pub struct InstagramUserList {
    pub users: Vec<InstagramUser>,
}
