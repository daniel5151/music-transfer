cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "windows.rs"]
        mod sys;
    } else {
        mod sys {
            pub struct VolumeControllerImpl;
            impl VolumeControllerImpl {
                pub fn new_system_default() -> anyhow::Result<Self> {
                    Err(anyhow::anyhow!("no volume controller is currently implemented for this platform. consider opening a PR?"))
                }

                pub fn get_master_volume(&self) -> anyhow::Result<f32> {
                    Ok(0.0)
                }

                pub fn set_master_volume(&self, _vol: f32) -> anyhow::Result<()> {
                    Ok(())
                }
            }
        }
    }
}

/// Control system audio
pub struct VolumeController(sys::VolumeControllerImpl);

impl VolumeController {
    /// Construct a new [`AudioCtl`] to control the system's default audio
    /// endpoint.
    pub fn new_system_default() -> anyhow::Result<Self> {
        Ok(VolumeController(
            sys::VolumeControllerImpl::new_system_default()?,
        ))
    }

    /// Get the master volume (0.0 = mute, 1.0 = max volume)
    pub fn get_master_volume(&self) -> anyhow::Result<f32> {
        self.0.get_master_volume().map_err(Into::into)
    }

    /// Set the master volume (0.0 = mute, 1.0 = max volume)
    pub fn set_master_volume(&self, vol: f32) -> anyhow::Result<()> {
        self.0.set_master_volume(vol).map_err(Into::into)
    }
}
