mod ui;
mod window;
pub use smithay_client_toolkit::shell::wlr_layer::Anchor;
pub use ui::{LayerShellPlatform, RgbaPixel};
pub use window::Bar;
pub type EventQueue = wayland_client::EventQueue<Bar>;
