use std::fs;

use futures_util::{StreamExt, stream::SplitStream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Serialize, Deserialize, Clone)]
struct AirtipsConfig {
    ws_server_address: String,
    airtips_server_address: String,
    devinlittlenet_address: String,
    devinlittlenet_username: String,
    devinlittlenet_password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_file =
        fs::read_to_string("./airtips_config.toml").expect("Should have a config.toml file");
    let tomlable: AirtipsConfig = toml::from_str(config_file.as_str()).unwrap();

    let token_config = tomlable.clone();

    let login_payload = json!({
        "username": token_config.devinlittlenet_username,
        "password": token_config.devinlittlenet_password,
    });

    let devin_login = reqwest::Client::new()
        .post(format!(
            "{}/gradegetter/auth/login",
            token_config.devinlittlenet_address
        ))
        .header("Content-Type", "application/json")
        .json(&login_payload)
        .send()
        .await?;

    let token = devin_login
        .text()
        .await
        .unwrap()
        .trim_matches('"')
        .to_string();

    let (ws_stream, _) = connect_async(tomlable.ws_server_address).await?;

    let (_write, read) = ws_stream.split();
    handle_messages(read, token).await
}

async fn handle_messages(
    mut read: WsRead,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(msg) = read.next().await {
        // HACK: CLONING TOKEN FOR EVERY REQUEST = BAD IDEA
        process_message(msg?, token.clone()).await.unwrap();
    }
    Ok(())
}

async fn process_message(msg: Message, token: String) -> Result<(), Box<dyn std::error::Error>> {
    match msg {
        Message::Text(text) => handle_text_message(&text, token).await,
        Message::Close(_) => handle_close(),
        _ => Ok(()),
    }
}

fn handle_close() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connection closed");
    Err("Connection closed by server".into())
}

// NOTE: PROCESSING!!

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "PLAYER_INFO")]
    PlayerInfo {
        song: Song,
        #[serde(rename = "isPlaying")]
        is_playing: bool,
        muted: bool,
        position: u64,
        volume: u8,
        repeat: String,
        shuffle: bool,
    },
    #[serde(rename = "POSITION_CHANGED")]
    PositionChanged { position: f64 },
    #[serde(rename = "PLAYER_STATE_CHANGED")]
    PlayerStateChanged {
        #[serde(rename = "isPlaying")]
        is_playing: bool,
        position: u64,
    },
    #[serde(rename = "VIDEO_CHANGED")]
    VideoChanged { song: Song, position: u64 },
    #[serde(rename = "VOLUME_CHANGED")]
    VolumeChanged { volume: u8, muted: bool },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Song {
    pub title: String,
    pub alternative_title: String,
    pub artist: String,
    pub artist_url: String,
    pub views: u64,
    pub image_src: String,
    //    pub image: Image,
    pub is_paused: bool,
    pub song_duration: u64,
    pub elapsed_seconds: u64,
    pub url: String,
    pub album: Option<String>,
    pub video_id: String,
    pub playlist_id: String,
    pub media_type: String,
    pub tags: Vec<String>,
}

async fn handle_text_message(text: &str, token: String) -> Result<(), Box<dyn std::error::Error>> {
    let message: WsMessage = serde_json::from_str(text)?;

    let config_file =
        fs::read_to_string("./airtips_config.toml").expect("Should have a config.toml file");
    let tomlable: AirtipsConfig = toml::from_str(config_file.as_str()).unwrap();

    match message {
        WsMessage::PlayerInfo { song, .. } => match post_song(song, tomlable, token).await {
            Ok(_) => println!("song status uppppdated!!!"),
            Err(e) => eprintln!("Failed to post song: {}, ... do better", e),
        },
        WsMessage::VideoChanged { song, .. } => match post_song(song, tomlable, token).await {
            Ok(_) => println!("song status uppppdated!!!"),
            Err(e) => eprintln!("Failed to post song: {}, ... do better", e),
        },
        WsMessage::PositionChanged { .. } => {}
        WsMessage::PlayerStateChanged { .. } => {}
        WsMessage::VolumeChanged { .. } => {}
    }

    Ok(())
}

async fn post_song(
    song: Song,
    config: AirtipsConfig,
    token: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let payload = song;

    let response = client
        .post(format!(
            "{}/airtips/post_song",
            config.airtips_server_address
        ))
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await?;

    if !response.status().to_string().contains("200") {
        println!("Status: {}", response.status());
        let body = response.text().await?;
        println!("Response: {}", body);
    }

    Ok(())
}
