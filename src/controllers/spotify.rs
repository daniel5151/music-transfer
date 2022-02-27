use anyhow::anyhow;
use rspotify::clients::OAuthClient;
use rspotify::scopes;
use rspotify::AuthCodeSpotify;
use rspotify::Credentials;
use rspotify::OAuth;

pub struct SpotifyWrapper {
    spotify: AuthCodeSpotify,
}

impl SpotifyWrapper {
    pub async fn new(
        cache_path: &str,
        client_id: &str,
        client_secret: &str,
        redirect_uri: &str,
    ) -> anyhow::Result<SpotifyWrapper> {
        let creds = Credentials::new(client_id, client_secret);
        let oauth = OAuth {
            redirect_uri: redirect_uri.to_string(),
            scopes: scopes!(
                "user-modify-playback-state",
                "user-read-playback-state",
                "user-read-recently-played"
            ),
            ..OAuth::default()
        };
        let mut spotify = AuthCodeSpotify::with_config(
            creds.clone(),
            oauth.clone(),
            rspotify::Config {
                cache_path: cache_path.into(),
                token_cached: true,
                token_refreshing: true,
                ..rspotify::Config::default()
            },
        );

        let url = spotify.get_authorize_url(false).unwrap();
        spotify
            .prompt_for_token(&url)
            .await
            .expect("couldn't authenticate successfully");

        Ok(SpotifyWrapper { spotify })
    }

    pub async fn devices(&self) -> anyhow::Result<Vec<rspotify::model::Device>> {
        self.spotify.device().await.map_err(Into::into)
    }

    pub async fn transfer_playback(&mut self, device: &str) -> anyhow::Result<()> {
        // translate the target device name to an id
        let devices = self.spotify.device().await?;
        let device = devices
            .into_iter()
            .find(|d| d.name == device)
            .ok_or_else(|| anyhow!("target device is not online"))?;
        let device_id = device
            .id
            .expect("/me/player/devices returned device with None id");

        let current_playback = self
            .spotify
            .current_playback(None, None::<std::slice::Iter<'_, _>>)
            .await?
            .expect("/me/playback response cannot be empty");

        log::info!(
            "transferring playback from {} to {}",
            current_playback.device.name,
            device.name
        );

        if current_playback.device.id.unwrap() == device_id {
            log::warn!("attempting to transfer playback to current device - doing nothing");
            return Ok(());
        }

        self.spotify.transfer_playback(&device_id, None).await?;

        Ok(())
    }
}
