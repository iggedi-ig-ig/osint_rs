use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum FollowUnfollowResult {
    Follow { result: String, status: String },
    Unfollow { status: String },
}

#[test]
pub fn test_deserialize_follow_unfollow_status() {
    let test_data = r#"[{ "result": "following", "status": "ok" }, { "status": "ok" }]"#;
    let test: Vec<FollowUnfollowResult> = serde_json::from_str(test_data).unwrap();
    println!("{:#?}", test);
}
