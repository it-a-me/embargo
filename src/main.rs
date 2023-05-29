use slint::{platform::software_renderer::MinimalSoftwareWindow, PhysicalSize, Rgb8Pixel};
use smithay_client_toolkit::shell::wlr_layer::Anchor;
use wayland_client::Connection;

mod error;
mod ui;
mod window;
slint::include_modules!();
//pub use error::Error;

fn main() -> anyhow::Result<()> {
    let window = MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );
    slint::platform::set_platform(Box::new(ui::BasicPlatform::new(window.clone()))).unwrap();
    let ui = MainUi::new()?;
    let mut count = 0;
    window.set_size(PhysicalSize::new(1920, 100));
    let conn = Connection::connect_to_env()?;
    let (mut bar, mut event_queue) = window::BarLayer::new(&conn, Anchor::TOP, 1920, 100)?;
    loop {
        ui.set_text(format!("{count}").into());
        event_queue.blocking_dispatch(&mut bar)?;
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            renderer.render(&mut bar.software_buffer, 1920);
        });
        ui.set_rect_x(if bar.clickies { 100.0 } else { 0.0 });
        count += 1;
        if bar.exit {
            break;
        }
    }
    Ok(())
}
