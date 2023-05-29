use std::{rc::Rc, time::Instant};

use slint::platform::{software_renderer::MinimalSoftwareWindow, Platform};
pub struct BasicPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: Instant,
}
impl BasicPlatform {
    pub fn new(window: Rc<MinimalSoftwareWindow>) -> Self {
        Self {
            window,
            start_time: Instant::now(),
        }
    }
}
impl Platform for BasicPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<std::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        self.start_time.elapsed()
    }
}
