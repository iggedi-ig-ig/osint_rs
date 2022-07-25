use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiUser {
    pub pk: u64,
    pub username: String,
    pub full_name: String,
    pub is_private: bool,
    pub profile_pic_url: String,
    pub is_verified: bool,
}

#[derive(Deserialize, Debug)]
pub struct TrayItem {
    pub id: u64,
    pub user: ApiUser,
}

#[derive(Deserialize, Debug)]
pub struct InstagramReelsTray {
    pub tray: Vec<TrayItem>,
}

#[test]
pub fn test_deserialize_reels() {
    let test_data = include_str!("../../../test_data/reel_tray.json");
    let instagram_reels_tray: InstagramReelsTray = serde_json::from_str(test_data).unwrap();
    println!("{:#?}", instagram_reels_tray);
}
