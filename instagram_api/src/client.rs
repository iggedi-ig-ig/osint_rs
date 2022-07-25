use crate::models::actions::FollowUnfollowResult;
use crate::models::explore::InstagramHashtag;
use crate::models::graph_ql::graph_ql_user::InstagramUserGraphQlData;
use crate::models::login::LoginResponse;
use crate::models::timeline::InstagramTimeline;
use crate::models::InstagramIdentifier;
use crate::models::*;
use crate::InstagramCredentials;
use log::{debug, info};
use regex::Regex;
use reqwest::{Client, ClientBuilder, RequestBuilder, Response, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

#[macro_export]
macro_rules! default_headers {
    () => {
        crate::headers! {
            "accept": "/",
            "accept-language": "de-DE,de;q=0.9,en-DE;q=0.8,en;q=0.7,en-US;q=0.6",
            "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36",
            "content-type": "application/x-www-form-urlencoded",
            "sec-ch-ua": r##"" Not;A Brand";v="99", "Google Chrome";v="97", "Chromium";v="97""##,
            "sec-ch-ua-mobile": "?0",
            "sec-ch-ua-platform": r#""Windows""#,
            "sec-fetch-dest": "empty",
            "sec-fetch-mode": "cors",
            "sec-fetch-site": "same-site",
            "x-asbd-id": "198387",
            "x-ig-app-id": "936619743392459",
            "x-ig-www-claim": "hmac.AR0URJl1JVmMqO_CkGLyWvIcPETMrMOdDdUT1BcS3xgZHN2V",
            "x-instagram-ajax": "6ab3c34e0025"
        }
    };
    ($csrf:expr) => {
        crate::headers! {
            "accept": "/",
            "accept-language": "de-DE,de;q=0.9,en-DE;q=0.8,en;q=0.7,en-US;q=0.6",
            "user-agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/97.0.4692.71 Safari/537.36",
            "content-type": "application/x-www-form-urlencoded",
            "sec-ch-ua": r##"" Not;A Brand";v="99", "Google Chrome";v="97", "Chromium";v="97""##,
            "sec-ch-ua-mobile": "?0",
            "sec-ch-ua-platform": r#""Windows""#,
            "sec-fetch-dest": "empty",
            "sec-fetch-mode": "cors",
            "sec-fetch-site": "same-site",
            "x-csrftoken": $csrf,
            "x-asbd-id": "198387",
            "x-ig-app-id": "936619743392459",
            "x-ig-www-claim": "hmac.AR0URJl1JVmMqO_CkGLyWvIcPETMrMOdDdUT1BcS3xgZHN2V",
            "x-instagram-ajax": "6ab3c34e0025"
        }
    };
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstagramSession {
    pub username: String,
    pub user_id: String,
    pub cookie_str: String,
}

impl InstagramSession {
    const CSRF_REGEX: &'static str = r#""csrf_token":"(?P<token>.*?)""#;

    const BASE_URL: &'static str = "https://www.instagram.com";
    const LOGIN_URL: &'static str = "https://www.instagram.com/accounts/login/ajax/";

    pub async fn create_session(
        credentials: InstagramCredentials<'_>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        info!("trying login with account {credentials}");
        let response = reqwest::get(Self::BASE_URL).await?;
        let response_text = response.text().await?;
        let csrf_token = Regex::new(Self::CSRF_REGEX)?
            .captures(&response_text)
            .ok_or("csrf token regex doesn't capture anything")?
            .name("token")
            .ok_or("couldn't find csrf token")?
            .as_str()
            .to_owned();

        debug!("got csrf token {csrf_token}");

        let mut cookies = String::new();
        let headers = default_headers!(&csrf_token);
        // TODO: maybe we can cache this cookie jar somehow
        let client = ClientBuilder::new()
            .cookie_store(true)
            .default_headers(headers)
            .build()?;

        let payload = [
            ("enc_password", &*credentials.enc_password()),
            ("username", &*credentials.username()),
            ("query_params", "{}"),
            ("opt_into_one_tap", "false"),
            ("stop_deleting_nonce", ""),
            ("trusted_device_records", "{}"),
        ];

        let raw_response = client.post(Self::LOGIN_URL).form(&payload).send().await?;
        for cookie in raw_response.cookies() {
            cookies.push_str(&format!("{}={}; ", cookie.name(), cookie.value()));
        }

        match raw_response.json().await? {
            LoginResponse::LoginSuccess {
                user_id,
                authenticated,
                ..
            } if authenticated => {
                info!("successfully logged into account {credentials}");

                Ok(Self {
                    username: credentials.username(),
                    user_id,
                    cookie_str: cookies,
                })
            }
            LoginResponse::LoginFail { message, .. } => Err(format!(
                "Failed to login to account {credentials}. Reason: {}",
                message
            )
            .into()),
            _ => Err("Failed to login for an unknown reason.".into()),
        }
    }
}

#[derive(Clone)]
pub struct InstagramClient {
    client: Client,
    user_id: InstagramIdentifier,
}

unsafe impl Send for InstagramClient {}
unsafe impl Sync for InstagramClient {}

#[async_trait::async_trait]
pub trait InstagramRequest<T> {
    async fn execute(self) -> Result<T, Box<dyn std::error::Error + Sync + Send>>;

    async fn execute_raw(self) -> Result<Response, Box<dyn std::error::Error + Sync + Send>>;
}

#[async_trait::async_trait]
impl<T> InstagramRequest<T> for RequestBuilder
where
    T: for<'de> Deserialize<'de>,
{
    async fn execute(self) -> Result<T, Box<dyn Error + Sync + Send>> {
        Ok(self.send().await?.json().await?)
    }

    async fn execute_raw(self) -> Result<Response, Box<dyn Error + Sync + Send>> {
        Ok(self.send().await?)
    }
}

impl InstagramClient {
    /// returns the user id of the currently logged in user
    pub fn user_id(&self) -> String {
        self.user_id.to_string()
    }

    /// tries to login to instagram using the given credentials and returns an InstagramClient instance if this succeeds
    pub fn from_session(
        session: &InstagramSession,
    ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut headers = default_headers!();
        headers.insert(reqwest::header::COOKIE, session.cookie_str.parse()?);

        Ok(Self {
            client: ClientBuilder::new()
                .cookie_store(true)
                .default_headers(headers)
                .build()?,
            user_id: InstagramIdentifier::Literal(session.user_id.clone()),
        })
    }

    const TIMELINE_URL: &'static str = "https://i.instagram.com/api/v1/feed/timeline";

    /// returns the users timeline
    pub fn get_timeline(&self) -> impl InstagramRequest<InstagramTimeline> {
        self.client.get(Self::TIMELINE_URL)
    }

    const USER_INFO_URL: &'static str = "https://www.instagram.com/$user$/?__a=1&__d";

    /// lets you query a certain user by their username
    pub fn get_user_data_graphql(
        &self,
        username: &str,
    ) -> impl InstagramRequest<InstagramUserGraphQlData> {
        self.client
            .get(Self::USER_INFO_URL.replace("$user$", username))
    }

    const POST_DATA_URL: &'static str =
        "https://www.instagram.com/graphql/query/?query_hash=8c2a529969ee035a5063f2fc8602a0fd";
    pub fn get_post_data_graphql(
        &self,
        post_id: &str,
    ) -> Result<impl InstagramRequest<Value>, Box<dyn std::error::Error + Sync + Send>> {
        Ok(self.client.get(Url::parse_with_params(
            Self::POST_DATA_URL,
            &[("id", &*post_id), ("first", "12")],
        )?))
    }

    const USER_FOLLOW_URL: &'static str =
        "https://www.instagram.com/web/friendships/$user$/follow/";

    pub fn follow_user(
        &self,
        user_id: InstagramIdentifier,
    ) -> impl InstagramRequest<FollowUnfollowResult> {
        self.client
            .get(Self::USER_FOLLOW_URL.replace("$user$", &*user_id.to_string()))
    }

    const USER_UNFOLLOW_URL: &'static str =
        "https://www.instagram.com/web/friendships/$user$/unfollow/";

    pub fn unfollow_user(
        &self,
        user_id: InstagramIdentifier,
    ) -> impl InstagramRequest<FollowUnfollowResult> {
        self.client
            .get(Self::USER_UNFOLLOW_URL.replace("$user$", &*user_id.to_string()))
    }

    const EXPLORE_TAG_URL: &'static str =
        "https://www.instagram.com/explore/tags/$hashtag$/?__a=1&__d=dis";

    pub fn explore_hashtag(&self, hashtag: &str) -> impl InstagramRequest<InstagramHashtag> {
        self.client
            .get(Self::EXPLORE_TAG_URL.replace("$hashtag$", hashtag))
    }

    const FRIENDSHIP_URL: &'static str =
        "https://i.instagram.com/api/v1/friendships/$userid$/$action$/?count=$count$";
    pub fn get_user_followers(
        &self,
        id: &str,
        count: u32,
    ) -> impl InstagramRequest<InstagramUserList> {
        self.client.get(
            Self::FRIENDSHIP_URL
                .replace("$userid$", id)
                .replace("$action$", "followers")
                .replace("$count$", &*count.to_string()),
        )
    }

    pub fn get_user_following(
        &self,
        id: &str,
        count: u32,
    ) -> impl InstagramRequest<InstagramUserList> {
        self.client.get(
            Self::FRIENDSHIP_URL
                .replace("$userid$", id)
                .replace("$action$", "following")
                .replace("$count$", &*count.to_string()),
        )
    }
}
