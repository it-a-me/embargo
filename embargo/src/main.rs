use clap::Parser;
use human_repr::HumanCount;
use slint::{
    platform::software_renderer::MinimalSoftwareWindow, ComponentHandle, ModelRc, PhysicalSize,
};
use wayland_client::Connection;

mod cli;
mod config;
mod error;
mod hardware_mon;
mod ui;
mod window;
slint::include_modules!();

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .without_time()
            .pretty()
            .with_max_level(args.log_level)
            .finish(),
    )?;
    let conf = config::Config::parse(args.override_config.as_deref())?;
    let (width, height) = (1920, 40);
    let window = MinimalSoftwareWindow::new(
        slint::platform::software_renderer::RepaintBufferType::ReusedBuffer,
    );
    slint::platform::set_platform(Box::new(ui::BasicPlatform::new(window.clone()))).unwrap();
    let ui = MainUi::new()?;
    #[cfg(feature = "hyprland")]
    ui.global::<Hyprland>()
        .on_change_workspace(|id| hyprland_workspaces::change_workspace(id).unwrap());
    window.set_size(PhysicalSize::new(width, height));
    let conn = Connection::connect_to_env()?;
    let (mut bar, mut event_queue) = window::Bar::new(
        &conn,
        window.clone(),
        ui::RgbaPixel::transparent(),
        conf.anchor,
        &conf.layer_name,
        width,
        height,
    )?;
    let mut hw_mon = hardware_mon::HardwareMonitor::new("enp6s0".into());
    hw_mon.update();
    #[cfg(feature = "hyprland")]
    let mut workspaces;
    let mut formatted_time;
    let mut time;
    ui.global::<HardwareMonitor>().set_totalmemory(
        HumanCount::human_count_bytes(hw_mon.total_mem())
            .to_string()
            .into(),
    );
    loop {
        event_queue.blocking_dispatch(&mut bar)?;
        slint::platform::update_timers_and_animations();
        hw_mon.update();
        #[cfg(feature = "hyprland")]
        {
            workspaces = hyprland::Workspaces::new()?;
        }

        time = chrono::Local::now();
        formatted_time = time.format("%I:%M%P -- %d of %b, %Y").to_string();

        #[cfg(feature = "hyprland")]
        ui.set_workspaces(ModelRc::new(workspaces.as_modal()));
        ui.set_time(formatted_time.into());
        ui.global::<HardwareMonitor>().set_cpu_usage(
            ((hw_mon.cpu_usage() * 10.0).round() / 10.0)
                .to_string()
                .into(),
        );
        ui.global::<HardwareMonitor>().set_used_memory(
            HumanCount::human_count_bytes(hw_mon.used_mem())
                .to_string()
                .into(),
        );
        ui.global::<HardwareMonitor>().set_network_up(
            human_repr::HumanThroughput::human_throughput_bytes(hw_mon.uploaded_bytes())
                .to_string()
                .into(),
        );
        ui.global::<HardwareMonitor>().set_network_down(
            human_repr::HumanThroughput::human_throughput_bytes(hw_mon.downloaded_bytes())
                .to_string()
                .into(),
        );
        window.draw_if_needed(|renderer| {
            renderer.render(&mut bar.software_buffer, width as usize);
        });
        if bar.exit {
            break;
        }
    }
    Ok(())
}
#[cfg(feature = "hyprland")]
pub mod hyprland {
    use hyprland_workspaces::{Workspace, WorkspaceState};
    use slint::{Color, VecModel};
    pub struct Workspaces(Vec<Workspace>);
    impl Workspaces {
        pub fn new() -> anyhow::Result<Self> {
            let workspaces = hyprland_workspaces::workspaces()?;
            Ok(Workspaces(workspaces))
        }
        pub fn as_modal(&self) -> VecModel<(Color, Color, i32)> {
            let workspaces = self
                .0
                .iter()
                .map(|w| {
                    //fkasjl
                    let color = Self::state_to_color(&w.state);
                    (color, color.brighter(0.2), w.id)
                })
                .collect::<Vec<_>>();
            VecModel::from(workspaces)
        }
        fn state_to_color(state: &WorkspaceState) -> Color {
            match state {
                WorkspaceState::Active => Color::from_rgb_u8(48, 112, 144),
                WorkspaceState::Used => Color::from_rgb_u8(32, 64, 80),
                WorkspaceState::Unused => Color::from_rgb_u8(50, 54, 63),
            }
        }
    }
}
