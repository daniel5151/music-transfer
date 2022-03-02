use windows::core::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub struct PowerControllerImpl {}

impl PowerControllerImpl {
    pub fn new_system_default() -> Result<PowerControllerImpl> {
        Ok(PowerControllerImpl {})
    }

    pub fn wake_screen(&self) -> anyhow::Result<()> {
        // pretty jank, but the little jiggle does the trick lol
        unsafe {
            mouse_event(MOUSEEVENTF_MOVE | MOUSEEVENTF_MOVE_NOCOALESCE, 0, 1, 0, 0);
            mouse_event(MOUSEEVENTF_MOVE | MOUSEEVENTF_MOVE_NOCOALESCE, 0, -1, 0, 0);
        }

        Ok(())
    }
}
