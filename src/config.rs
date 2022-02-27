use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub spotify_creds: Option<SpotifyCreds>,
    pub spotify_transfer: Option<SpotifyTransfer>,
    pub volume_sync: Option<VolumeSync>,
}

#[derive(Serialize, Deserialize)]
pub struct SpotifyCreds {
    pub spotify_client_id: String,
    pub spotify_client_secret: String,
    pub spotify_redirect_uri: String,
}

#[derive(Serialize, Deserialize)]
pub struct SpotifyTransfer {
    pub spotify_name_remote: String,
    pub spotify_name_local: String,
}

#[derive(Serialize, Deserialize)]
pub struct VolumeSync {
    pub remote_host: String,
    pub remote_port: u16,
}
