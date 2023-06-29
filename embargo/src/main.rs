use clap::Parser;
use slint::{platform::software_renderer::MinimalSoftwareWindow, ComponentHandle, PhysicalSize};
mod cli;
mod config;
mod error;
mod hardware_mon;
mod run;
pub type Window = std::rc::Rc<MinimalSoftwareWindow>;
slint::include_modules!();
use layer_platform::{Bar, LayerShellPlatform, RgbaPixel};

fn setup_logger(
    log_level: tracing::Level,
) -> Result<(), tracing::subscriber::SetGlobalDefaultError> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .without_time()
            .pretty()
            .with_max_level(log_level)
            .finish(),
    )
}

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    setup_logger(args.log_level)?;

    let conf = config::Config::parse(args.override_config.as_deref())?;

    let (width, height) = (1920, 40);
    let window = MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );
    slint::platform::set_platform(Box::new(LayerShellPlatform::new(window.clone()))).unwrap();
    let ui = MainUi::new()?;
    #[cfg(feature = "hyprland")]
    ui.global::<Workspaces>()
        .on_change_workspace(|id| hyprland_workspaces::change_workspace(id as u32).unwrap());
    window.set_size(PhysicalSize::new(width, height));
    let (bar, event_queue) = Bar::new(
        window.clone(),
        RgbaPixel::transparent(),
        conf.anchor,
        &conf.layer_name,
        width,
        height,
    )?;
    run::run(ui, bar, event_queue, window, width)
}
