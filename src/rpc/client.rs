use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

// I don't want to hear a _word_ about this current protocol. it works fiiiiine

pub struct AudioClient {
    socket: TcpStream,
}

impl AudioClient {
    pub async fn new(remote_host: String, remote_port: u16) -> anyhow::Result<AudioClient> {
        let remote_addr = format!("{}:{}", remote_host, remote_port);

        // for some reason, passing a `foo.local` address directly to TcpStream::connect
        // is _super_ slow on windows box. Instead, we preemptively resolve any
        // `.local` addresses to a ipv4 address (falling back to the default resolution
        // if no ipv4 address could be found)
        let socket = if remote_host.ends_with(".local") {
            match tokio::net::lookup_host(remote_addr.clone())
                .await?
                .find(|addr| addr.is_ipv4())
            {
                Some(addr) => TcpStream::connect(addr).await?,
                None => TcpStream::connect(remote_addr).await?,
            }
        } else {
            TcpStream::connect(remote_addr).await?
        };

        Ok(AudioClient { socket })
    }

    pub async fn set_remote_volume(&mut self, vol: f32) -> anyhow::Result<()> {
        self.socket
            .write_all(format!("s:{}", vol).as_bytes())
            .await?;

        Ok(())
    }

    pub async fn get_remote_volume(&mut self) -> anyhow::Result<f32> {
        self.socket.write_all("g:".as_bytes()).await?;

        let mut cmd: [u8; 2] = [0; 2];
        self.socket.read_exact(&mut cmd).await?;

        if cmd != *b"G:" {
            return Err(anyhow::anyhow!("malformed response"));
        }

        let mut new_vol = String::new();
        self.socket.read_to_string(&mut new_vol).await?;
        let new_vol = new_vol.parse::<f32>()?;

        Ok(new_vol)
    }
}
