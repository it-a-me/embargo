use std::{rc::Rc, time::Instant};

use slint::platform::{
    software_renderer::{MinimalSoftwareWindow, TargetPixel},
    Platform,
};
pub struct LayerShellPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: Instant,
}
impl LayerShellPlatform {
    pub fn new(window: Rc<MinimalSoftwareWindow>) -> Self {
        Self {
            window,
            start_time: Instant::now(),
        }
    }
}
impl Platform for LayerShellPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<std::rc::Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.window.clone())
    }
    fn duration_since_start(&self) -> core::time::Duration {
        self.start_time.elapsed()
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RgbaPixel {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl RgbaPixel {
    pub fn transparent() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            alpha: 0,
        }
    }
}
impl TargetPixel for RgbaPixel {
    fn from_rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha: u8::MAX,
        }
    }

    fn blend(&mut self, color: slint::platform::software_renderer::PremultipliedRgbaColor) {
        let alpha = (u8::MAX - color.alpha) as u16;
        self.red = (self.red as u16 * alpha / 255) as u8 + color.red;
        self.green = (self.green as u16 * alpha / 255) as u8 + color.green;
        self.blue = (self.blue as u16 * alpha / 255) as u8 + color.blue;
    }
}
impl Default for RgbaPixel {
    fn default() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
            alpha: u8::MAX,
        }
    }
}
