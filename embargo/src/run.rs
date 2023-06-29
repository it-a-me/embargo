use crate::hardware_mon;
use human_repr::HumanCount;
use layer_platform::Bar;
use slint::{platform::software_renderer::MinimalSoftwareWindow, ComponentHandle};
use slint_interpreter::ComponentInstance;
pub fn run(
    // ui: MainUi,
    ui: &ComponentInstance,
    mut bar: Bar,
    mut event_queue: layer_platform::EventQueue,
    window: &std::rc::Rc<MinimalSoftwareWindow>,
    width: u32,
) -> anyhow::Result<()> {
    let mut hw_mon = hardware_mon::HardwareMonitor::new("enp6s0".into());
    hw_mon.update();
    #[cfg(feature = "hyprland")]
    let mut workspaces;
    let mut formatted_time;
    let mut time;
    ui.set_global_property(
        "HardwareMonitor",
        "totalmemory",
        slint_interpreter::Value::String(
            HumanCount::human_count_bytes(hw_mon.total_mem())
                .to_string()
                .into(),
        ),
    )?;
    ui.show()?;
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
        // ui.set_workspaces());
        ui.set_property("workspaces", workspaces.as_value())?;
        ui.set_property(
            "time",
            slint_interpreter::Value::String(formatted_time.into()),
        )?;
        ui.set_global_property(
            "HardwareMonitor",
            "cpu_usage",
            slint_interpreter::Value::String(
                ((hw_mon.cpu_usage() * 10.0).round() / 10.0)
                    .to_string()
                    .into(),
            ),
        )?;
        ui.set_global_property(
            "HardwareMonitor",
            "network_up",
            slint_interpreter::Value::String(
                human_repr::HumanThroughput::human_throughput_bytes(hw_mon.uploaded_bytes())
                    .to_string()
                    .into(),
            ),
        )?;
        ui.set_global_property(
            "HardwareMonitor",
            "network_down",
            slint_interpreter::Value::String(
                human_repr::HumanThroughput::human_throughput_bytes(hw_mon.downloaded_bytes())
                    .to_string()
                    .into(),
            ),
        )?;
        ui.set_global_property(
            "HardwareMonitor",
            "used_memory",
            slint_interpreter::Value::String(
                HumanCount::human_count_bytes(hw_mon.used_mem())
                    .to_string()
                    .into(),
            ),
        )?;
        window.draw_if_needed(|renderer| {
            renderer.render(&mut bar.software_buffer, width as usize);
        });
        if bar.exit {
            break;
        }
    }
    Ok(())
}
#[cfg(feature = "workspaces")]
pub mod hyprland {
    use embargo_workspace::WorkspaceState;
    use hyprland_workspaces::HyprlandWorkspace as DisplayWorkspace;
    use slint::{private_unstable_api::re_exports::Color, Brush, VecModel};
    use slint_interpreter::{Struct, Value};
    pub struct Workspaces(Vec<DisplayWorkspace>);
    impl Workspaces {
        pub fn new() -> anyhow::Result<Self> {
            let workspaces = hyprland_workspaces::workspaces()?;
            Ok(Workspaces(workspaces))
        }
        pub fn as_value(&self) -> Value {
            let workspaces = self
                .0
                .iter()
                .map(|w| {
                    //fkasjl
                    let color = Self::state_to_color(w.state);
                    Struct::from_iter([
                        ("color".into(), Brush::from(color).into()),
                        ("hover_color".into(), Brush::from(color).into()),
                        ("id".into(), w.position.into()),
                        // ("hover_color".into(), color.brighter(0.2)),
                    ])
                    .into()
                })
                .collect::<Vec<_>>();
            let rc_modal = std::rc::Rc::new(VecModel::from(workspaces));
            let modalrc: slint::ModelRc<Value> = rc_modal.into();
            Value::Model(modalrc)
        }
        fn state_to_color(state: WorkspaceState) -> Color {
            match state {
                WorkspaceState::Active => Color::from_rgb_u8(48, 112, 144),
                WorkspaceState::Used => Color::from_rgb_u8(32, 64, 80),
                WorkspaceState::Unused => Color::from_rgb_u8(50, 54, 63),
            }
        }
    }
}
