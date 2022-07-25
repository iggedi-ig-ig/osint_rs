use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct LoginSpamData {
    pub spam: bool,
    pub feedback_title: String,
    pub feedback_message: String,
    pub feedback_url: String,
    pub feedback_appeal_label: String,
    pub feedback_ignore_label: String,
    pub feedback_action: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "status")]
pub enum LoginResponse {
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "ok")]
    LoginSuccess {
        user: bool,
        user_id: String,
        authenticated: bool,
        one_tap_prompt: bool,
    },
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "fail")]
    LoginFail {
        message: String,
        #[serde(default)]
        #[serde(flatten)]
        spam_data: Option<LoginSpamData>,
    },
}
