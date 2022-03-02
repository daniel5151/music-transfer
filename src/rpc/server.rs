use anyhow::Context;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

// I don't want to hear a _word_ about this current protocol. it works fiiiiine

pub struct AudioServer {
    port: u16,
}

impl AudioServer {
    pub fn new(port: u16) -> AudioServer {
        AudioServer { port }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let audio = crate::controllers::volume::VolumeController::new_system_default()
            .context("failed to init system volume controller")?;
        let power = crate::controllers::power::PowerController::new_system_default()
            .context("failed to init system power controller")?;

        let addr = format!("0.0.0.0:{}", self.port);
        log::info!("binding `music-transfer` server to {}", addr);

        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (mut socket, addr) = listener.accept().await?;
            log::info!("accepted connection from {}", addr);

            let mut cmd: [u8; 2] = [0; 2];
            socket.read_exact(&mut cmd).await?;

            if cmd[1] != b':' {
                return Err(anyhow::anyhow!("malformed command"));
            }

            log::info!("incoming cmd: {}", cmd[0] as char);

            match cmd[0] {
                b's' => {
                    let mut new_vol = String::new();
                    socket.read_to_string(&mut new_vol).await?;

                    log::info!("setting volume to: {}", new_vol);

                    let new_vol = new_vol
                        .parse::<f32>()
                        .context("invalid volume sent from client")?;

                    audio.set_master_volume(new_vol)?;
                    power.wake_screen()?;
                }
                b'g' => {
                    let current_volume = audio.get_master_volume()?;
                    log::info!("returning current volume: {}", current_volume);

                    socket
                        .write_all(format!("G:{}", current_volume).as_bytes())
                        .await?;
                }
                _ => return Err(anyhow::anyhow!("invalid command")),
            }
        }
    }
}
