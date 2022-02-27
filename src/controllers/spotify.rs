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

    pub async fn devices(&self) -> anyhow::Result<Vec<DeviceNormalized>> {
        let mut devices = Vec::new();
        for d in self.spotify.device().await? {
            devices.push(d.normalize()?)
        }
        Ok(devices)
    }

    pub async fn transfer_playback(
        &mut self,
        target_device_name: &str,
        sync_volume: bool,
    ) -> anyhow::Result<()> {
        let devices = self.devices().await?;
        let target_device = devices
            .into_iter()
            .find(|d| d.name == target_device_name)
            .ok_or_else(|| anyhow!("target device is not online"))?;

        let current_playback = self
            .spotify
            .current_playback(None, None::<std::slice::Iter<'_, _>>)
            .await?
            .expect("/me/playback response cannot be empty");

        let current_device = current_playback.device.normalize()?;

        if current_device.id == target_device.id {
            log::warn!("attempting to transfer playback to current device - doing nothing");
            return Ok(());
        }

        log::info!(
            "transferring playback from {} to {}",
            current_device.name,
            target_device.name
        );

        self.spotify
            .transfer_playback(&target_device.id, None)
            .await?;

        if sync_volume {
            log::info!(
                "matching volume from {} to {}",
                current_device.name,
                target_device.name
            );

            // so, what's with this janky loop?
            //
            // well, I wasn't able to get the volume API to work by explicitly specifying a
            // target device ID, and instead have to rely on the other behavior of "change
            // the volume of the currently playing device"
            //
            // unfortunately, the spotify API isn't instant, so after we've changed playback
            // devices on our end, it takes some time for the spotify backend to "catch up",
            // which requires a little bit of polling

            // bound the loop, in case I'm a bad programmer
            for _ in 0..10 {
                // gotta delay a bit...
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;

                let new_playback = self
                    .spotify
                    .current_playback(None, None::<std::slice::Iter<'_, _>>)
                    .await?
                    .expect("/me/playback response cannot be empty");

                let new_playback_device = new_playback.device.normalize()?;

                if new_playback_device.name == target_device.name
                    && new_playback_device.volume_percent == current_device.volume_percent
                {
                    // passing None uses the currently playing device, which we've now asserted is
                    // indeed the target device
                    self.spotify
                        .volume(current_device.volume_percent, None)
                        .await?;

                    break;
                }
            }
        }

        Ok(())
    }
}

// the rspotify device object assumes some fields can be nullable, when they
// really can't
#[derive(Debug)]
pub struct DeviceNormalized {
    pub id: String,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    pub _type: rspotify::model::DeviceType,
    pub volume_percent: u8,
}

trait SpotifyNormalize {
    type Normalized;
    fn normalize(self) -> anyhow::Result<Self::Normalized>;
}

impl SpotifyNormalize for rspotify::model::Device {
    type Normalized = DeviceNormalized;

    fn normalize(self) -> anyhow::Result<DeviceNormalized> {
        Ok(DeviceNormalized {
            id: self
                .id
                .ok_or_else(|| anyhow!("/me/player/devices returned device with None id"))?,
            is_active: self.is_active,
            is_private_session: self.is_private_session,
            is_restricted: self.is_restricted,
            name: self.name,
            _type: self._type,
            volume_percent: self.volume_percent.ok_or_else(|| {
                anyhow!("/me/player/devices returned device with None volume_percent")
            })? as u8,
        })
    }
}
