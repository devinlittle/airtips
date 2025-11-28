use futures_util::{StreamExt, stream::SplitStream};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type WsRead = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // TODO: CONFIG THISSSSS
    let url = "ws://0.0.0.0:26538/api/v1/ws";

    let (ws_stream, _) = connect_async(url).await?;
    println!("Connected to {}", url);

    let (_write, read) = ws_stream.split();
    handle_messages(read).await
}

async fn handle_messages(mut read: WsRead) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(msg) = read.next().await {
        process_message(msg?).await.unwrap();
    }
    Ok(())
}

async fn process_message(msg: Message) -> Result<(), Box<dyn std::error::Error>> {
    match msg {
        Message::Text(text) => handle_text_message(&text).await,
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

async fn handle_text_message(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let message: WsMessage = serde_json::from_str(text)?;

    match message {
        WsMessage::PlayerInfo { song, .. } => {
            if post_song(song).await.is_ok() {
                println!("We updated song status!");
            } else {
                println!("UHHOHH FAILEDD");
            }
        }
        WsMessage::VideoChanged { song, .. } => {
            if post_song(song).await.is_ok() {
                println!("We updated song status!");
            } else {
                println!("UHHOHH FAILEDD");
            }
        }
        WsMessage::PositionChanged { .. } => {}
        WsMessage::PlayerStateChanged { .. } => {}
        WsMessage::VolumeChanged { .. } => {}
    }

    Ok(())
}

async fn post_song(song: Song) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // TODO: GET RID OF THIS
    let token = "";

    let payload = song;

    // TODO: READ URL OFF OF CONFIG
    let response = client
        .post("http://10.0.0.139:3013/post_song")
        .header("Authorization", format!("Bearer {}", token))
        .json(&payload)
        .send()
        .await?;

    println!("Status: {}", response.status());

    let body = response.text().await?;
    println!("Response: {}", body);

    Ok(())
}
