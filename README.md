# `music-transfer`

A small utility to seamlessly transfer music playback between two computers.

Before you get too excited - no, there isn't any fancy over-the-network music
streaming going on. `music-transfer` is basically just a wrapper around the
Spotify Connect API (for music handoff) + a simple client/server app to sync
system volume.

`music-transfer` was designed with
[`display-switch`](https://github.com/haimgel/display-switch) in mind, as I
wanted seamless spotify handoff between my work/personal PC.

* * *

**NOTE: At the moment, cross-computer volume sync is only implemented on
Windows!**

If you'd like to contribute volume sync functionality for MacOS/Linux/etc, PRs
are more than welcome!

* * *

## Example: Using `music-transfer` alongside `display-switch`

As mentioned earlier, `music-transfer` was designed to integrate nicely with
[`display-switch`](https://github.com/haimgel/display-switch).

Here's a copy of the `display-switch.ini` that I use to achieve seamless music
handoff as part of the KVM switch.

```ini
usb_device = "1532:005C"
on_usb_connect = "DisplayPort1"
on_usb_disconnect = "Hdmi1"

on_usb_connect_execute = "'C:\\Users\\daprilik\\bin\\music-transfer.exe' --config-path 'C:\\Users\\daprilik\\bin\\music_transfer_config.json' --spotify-token-cache-path 'C:\\Users\\daprilik\\bin\\.spotify_token_cache.json' transfer local --sync-volume --spotify"
on_usb_disconnect_execute = "'C:\\Users\\daprilik\\bin\\music-transfer.exe' --config-path 'C:\\Users\\daprilik\\bin\\music_transfer_config.json' --spotify-token-cache-path 'C:\\Users\\daprilik\\bin\\.spotify_token_cache.json' transfer remote --sync-volume --spotify"
```

Note that you _could_ avoid explicitly passing `--config-path` and
`--spotify-token-cache-path` by figuring out which working directory
`display-switch.exe` is run from, but I prefer to be explicit over implicit.

## Setup

Run `cargo build --release`, and a binary will pop out at
`<repo>/target/release/music-transfer`.

### Spotify Authentication

The first time you want to do something spotify related with `music-transfer`,
you'll need to go through the Spotify authentication flow.

I suggest running `music-transfer list-spotify-devices`, as that's a quick and
easy way to validate that you can talk to the Spotify API.

After running this command, a browser window will pop up asking you to login to
Spotify. Upon successful login, you'll be redirected to a webpage specified in
`"spotify_redirect_uri"`, the URL of which you'll need to copy-paste into the
CLI window.

At this point, you *should* see a list of Spotify Connect devices get listed.

If that didn't work, double-check that you've set up your Spotify developer
account properly (as discussed under the [`spotify_creds`](#spotifycreds)
section below).

### Volume Sync - Remote Server

You'll want to set `music-transfer audio-server` to auto-launch at startup.

#### Windows

- Create a shortcut to the `music-transfer.exe`
- Open the shortcut's Properties, and modify the "target" to include the
  following CLI params: `audio-server --port 12345`
- Open Run (Win + R), and run `shell:startup`
- Drag the shortcut into that folder

The music-transfer audio server will now launch automatically at startup.

## Configuration

Depending on what features you're using, you'll need to fill out different/all
parts of the configuration file. It's pretty obvious what features correspond to
what settings, and if something isn't right, you should get a helpful error
message telling you what you're missing.

Notably, `volume-server` does _not_ require any config options to be set, as it
is entirely configured via the CLI.

```json
{
    "spotify_creds": {
        "spotify_client_id": "00000000000000000000000000000000",
        "spotify_client_secret": "00000000000000000000000000000000",
        "spotify_redirect_uri": "https://example.com/whatever"
    },
    "spotify_transfer": {
        "spotify_name_remote": "REMOTE_COMUTER_NAME",
        "spotify_name_local": "LOCAL_COMPUTER_NAME"
    },
    "volume_sync": {
        "remote_host": "REMOTE_COMUTER_NAME.local",
        "remote_port": "12345"
    }
}
```

### `spotify_creds`

In order to connect to the Spotify API, you'll need to register for a Spotify
developer account and provide `music-transfer` with a few bits of information.

See <https://docs.rs/rspotify/0.11.3/rspotify/#authorization> for details.

### `spotify_transfer`

- `spotify_name_remote`: friendly Spotify name for the remote computer
- `spotify_name_local`: friendly Spotify name for the local computer

_Note:_ you can use `music-transfer list-spotify-devices` to list available
spotify connect devices.

### `volume_sync`

In order to sync volume across computers, the remote computer needs to be
running an instance of `music-transfer audio-server`.

- `remote_host`: Hostname of remote computer (e.g: IP address, `.local` addr)
- `remote_port`: Port to connect to on the remote computer

