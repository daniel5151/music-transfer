cfg_if::cfg_if! {
    if #[cfg(windows)] {
        #[path = "windows.rs"]
        mod sys;
    } else {
        mod sys {
            pub struct PowerControllerImpl;
            impl PowerControllerImpl {
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
pub struct PowerController(sys::PowerControllerImpl);

impl PowerController {
    /// Construct a new [`PowerController`] to control the system's default
    /// audio endpoint.
    pub fn new_system_default() -> anyhow::Result<Self> {
        Ok(PowerController(
            sys::PowerControllerImpl::new_system_default()?,
        ))
    }

    /// Wake the screen (in case it was in standby mode)
    pub fn wake_screen(&self) -> anyhow::Result<()> {
        self.0.wake_screen()
    }
}
