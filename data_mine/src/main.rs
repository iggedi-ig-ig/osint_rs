mod ui;

use crate::ui::{ctrl_c_hook, App};
use clap::Parser;
use image::{DynamicImage, EncodableLayout};
use instagram_api::client::InstagramRequest;
use instagram_api::client::{InstagramClient, InstagramSession};
use instagram_api::InstagramCredentials;
use log::{info, LevelFilter};
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::panic;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// path to a file that contains username:password combinations of instagram accounts.
    #[clap(short = 'c', long = "combo", value_parser)]
    combo_path: PathBuf,
}

pub async fn download_image(
    url: &str,
) -> Result<DynamicImage, Box<dyn std::error::Error + Sync + Send>> {
    info!("downloading image {}", url);
    let response = reqwest::get(url).await?.bytes().await?;
    Ok(image::load_from_memory(response.as_bytes())?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Sync + Send>> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();

    panic::set_hook(Box::new(|info| {
        ui::ui_panic_hook(info);
    }));

    let cli: Cli = Cli::parse();

    let mut buff = String::new();
    let file = File::options()
        .read(true)
        .open(dbg!(cli.combo_path.as_path()))?;
    let mut reader = BufReader::new(file);

    let mut logged_in = Vec::new();
    while let Ok(len) = reader.read_line(&mut buff) {
        if len == 0 {
            break;
        }

        if let Some((username, password)) = buff.split_once(':') {
            let credentials = InstagramCredentials::new(username.trim(), password.trim());
            let session = InstagramSession::create_session(credentials).await.unwrap();
            logged_in.push(InstagramClient::from_session(&session)?);
        }
        buff.clear();
    }

    println!(
        "{:?}",
        logged_in
            .iter()
            .map(|client| client.user_id())
            .collect::<Vec<_>>()
    );

    let app = Arc::new(Mutex::new(App::new(
        vec![],
        logged_in.clone(),
        Vec::new(),
        Vec::new(),
    )));

    let app_ref = app.clone();
    let handle = tokio::spawn(async move { ui::setup_ui(&app_ref).unwrap() });

    ctrlc::set_handler(ctrl_c_hook).unwrap();

    //TODO: invent a cool discovery strategy for mapping user faces and so on...
    for client in logged_in {
        let mut app = app.lock().unwrap();

        let hashtag = "Sydney";
        let tag = client.explore_hashtag(hashtag).execute().await.unwrap();
        app.log(format!("Exploring hashtag {hashtag}"));
        for section in tag.data.top.sections {
            for x in section.layout_content.medias {
                let location = x.media.location;
                if let Some(location) = location {
                    app.add_location(location.clone());
                    app.log(format!(
                        "Location: {} lat: {}, lng: {}",
                        location.short_name, location.lat, location.lng
                    ));
                }
            }
        }
    }
    handle.await.unwrap();
    Ok(())
}
