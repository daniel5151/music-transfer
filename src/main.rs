#![windows_subsystem = "windows"]

use anyhow::Context;
use clap::Parser;
use clap::Subcommand;
use std::path::PathBuf;

mod config;
mod controllers;
mod rpc;

#[derive(Debug)]
enum SyncAudioTo {
    Remote,
    Local,
}

impl std::str::FromStr for SyncAudioTo {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "local" => SyncAudioTo::Local,
            "remote" => SyncAudioTo::Remote,
            _ => return Err("sync-audio-from must be one of 'local' or 'remote'"),
        };
        Ok(res)
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(long, default_value = "./music_transfer_config.json")]
    config_path: PathBuf,

    #[clap(long, default_value = "./.spotify_token_cache.json")]
    spotify_token_cache_path: String,

    #[clap(subcommand)]
    cmd: Command,
}

/// CLI utility for performing misc. spotify actions
#[derive(Debug, Subcommand)]
enum Command {
    /// Transfer audio playback + settings between two computers.
    Transfer {
        /// Which computer to transfer audio to.
        #[clap(possible_values = ["local", "remote"])]
        target: SyncAudioTo,

        /// Transfer spotify playback.
        #[clap(long)]
        spotify: bool,

        /// Sync volume between both computers.
        #[clap(long)]
        sync_volume: bool,
    },
    /// Utility: list all currently available spotify devices.
    ListSpotifyDevices,
    /// Start listening for incoming audio sync events.
    AudioServer {
        /// Port to listen on.
        #[clap(long)]
        port: u16,
    },
}

/// On Windows, re-attach the console if parent process has the console.
/// This allows to see the log output when run from the command line.
fn attach_console() {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::System::Console::*;
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    attach_console();

    env_logger::builder().parse_filters("info").init();

    let cli = Cli::parse();

    let config = {
        let res = tokio::fs::read_to_string(&cli.config_path)
            .await
            .context(format!(
                "failed to open config file ({:?})",
                cli.config_path
            ));

        match res {
            Ok(s) => {
                serde_json::from_str::<config::Config>(&s).context("could not parse config file")?
            }
            Err(e) => {
                if matches!(cli.cmd, Command::AudioServer { .. }) {
                    // it's fine if the config file couldn't be read, since the audio server doesn't
                    // need anything from it.
                    config::Config {
                        spotify_creds: None,
                        spotify_transfer: None,
                        volume_sync: None,
                    }
                } else {
                    return Err(e);
                }
            }
        }
    };

    match cli.cmd {
        Command::ListSpotifyDevices => {
            let config::SpotifyCreds {
                spotify_client_id,
                spotify_client_secret,
                spotify_redirect_uri,
            } = {
                config
                    .spotify_creds
                    .ok_or_else(|| anyhow::anyhow!(r#"missing "spotify_creds" from config"#))?
            };

            let spotify = controllers::spotify::SpotifyWrapper::new(
                &cli.spotify_token_cache_path,
                &spotify_client_id,
                &spotify_client_secret,
                &spotify_redirect_uri,
            )
            .await?;

            println!("{:#?}", spotify.devices().await?)
        }
        Command::AudioServer { port } => rpc::server::AudioServer::new(port).run().await?,
        Command::Transfer {
            target,
            spotify,
            sync_volume,
        } => {
            if !spotify && !sync_volume {
                log::warn!("executed 'tranfer' without including transfer option. doing nothing...")
            }

            if sync_volume {
                // make sure we can actually do system audio control before doing any networking
                let audio = controllers::volume::VolumeController::new_system_default()
                    .context("could not init system volume controller")?;

                let config::VolumeSync {
                    remote_host,
                    remote_port,
                } = {
                    config
                        .volume_sync
                        .ok_or_else(|| anyhow::anyhow!(r#"missing "volume_sync" from config"#))?
                };

                let mut client = rpc::client::AudioClient::new(remote_host, remote_port).await?;

                match target {
                    SyncAudioTo::Local => {
                        let new_vol = client
                            .get_remote_volume()
                            .await
                            .context("error communicating with remote server")?;

                        log::info!("setting local volume to {}", new_vol);

                        audio.set_master_volume(new_vol)?;
                    }
                    SyncAudioTo::Remote => {
                        let current_volume = audio.get_master_volume()?;

                        log::info!("setting remote volume to {}", current_volume);

                        client
                            .set_remote_volume(current_volume)
                            .await
                            .context("error communicating with remote server")?;
                    }
                }

                drop(client);
            }

            if spotify {
                let config::SpotifyCreds {
                    spotify_client_id,
                    spotify_client_secret,
                    spotify_redirect_uri,
                } = {
                    config
                        .spotify_creds
                        .ok_or_else(|| anyhow::anyhow!(r#"missing "spotify_creds" from config"#))?
                };

                let config::SpotifyTransfer {
                    spotify_name_remote,
                    spotify_name_local,
                } = {
                    config
                        .spotify_transfer
                        .ok_or_else(|| anyhow::anyhow!(r#"missing "spotify_creds" from config"#))?
                };

                let mut spotify = controllers::spotify::SpotifyWrapper::new(
                    &cli.spotify_token_cache_path,
                    &spotify_client_id,
                    &spotify_client_secret,
                    &spotify_redirect_uri,
                )
                .await?;

                spotify
                    .transfer_playback(
                        match target {
                            SyncAudioTo::Local => &spotify_name_local,
                            SyncAudioTo::Remote => &spotify_name_remote,
                        },
                        sync_volume,
                    )
                    .await?
            }
        }
    };

    Ok(())
}
