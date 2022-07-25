use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct InstagramUserGraphQlData {
    pub graphql: GraphQlData,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GraphQlData {
    pub user: GraphQlUser,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GraphQlUser {
    pub id: String,
    pub username: String,
    #[serde(rename = "edge_owner_to_timeline_media")]
    pub media_timeline: MediaTimeline,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaTimeline {
    pub count: u32,
    pub edges: Vec<Edge<MediaNode>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaNode {
    pub id: String,
    pub dimensions: MediaDimensions,
    pub display_url: String,
    #[serde(rename = "edge_media_to_tagged_user")]
    pub tagged_users: TaggedUserList,
    #[serde(rename = "edge_sidecar_to_children")]
    pub sidecar_children: Option<SidecarChildren>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct SidecarChildren {
    pub edges: Vec<Edge<MediaNode>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TaggedUserList {
    pub edges: Vec<Edge<TaggedUser>>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct MediaDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TaggedUser {
    pub user: TaggedUserInfo,
    pub x: f32,
    pub y: f32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TaggedUserInfo {
    pub username: String,
    pub full_name: String,
    pub id: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Edge<T> {
    pub node: T,
}

#[test]
pub fn test_deserialize_userdata() {
    let test_data = include_str!("../../../../test_data/userdata.json");

    let user_data: InstagramUserGraphQlData = serde_json::from_str(test_data).unwrap();
    println!("{:#?}", user_data);
}
