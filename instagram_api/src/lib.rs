use std::fmt::{Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

pub mod client;
pub mod models;

#[derive(Copy, Clone, Debug)]
pub struct InstagramCredentials<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

impl InstagramCredentials<'static> {
    pub fn new_const(username: &'static str, password: &'static str) -> Self {
        Self { username, password }
    }
}

impl<'a> InstagramCredentials<'a> {
    pub fn new(username: &'a str, password: &'a str) -> Self {
        Self { username, password }
    }

    pub fn username(&self) -> String {
        self.username.to_owned()
    }

    // TODO: this is deprecated and might not be supported for much longer
    pub fn enc_password(&self) -> String {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        format!(
            "#PWD_INSTAGRAM_BROWSER:0:{}:{}",
            time.as_secs(),
            self.password
        )
    }
}

impl<'a> Display for InstagramCredentials<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{}",
            self.username,
            (0..self.password.len()).map(|_| "*").collect::<String>()
        )
    }
}

#[macro_export]
macro_rules! headers {
    ($($key:literal: $value:expr),*) => {
        {
            let mut headers = reqwest::header::HeaderMap::new();
            $(
                headers.insert(reqwest::header::HeaderName::from_static($key), reqwest::header::HeaderValue::from_str($value).expect(&format!("Failed to parse header {} with value {}", $key, $value)));
            )*
            headers
        }
    };
}

#[test]
pub fn test_display_account() {
    let test_account = InstagramCredentials::new_const("test_username", "test password");
    println!("{}", test_account);
}
