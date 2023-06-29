use std::rc::Rc;
use tracing::{event, Level};

use crate::ui::RgbaPixel;
use crate::EventQueue;
use slint::{
    platform::{software_renderer::MinimalSoftwareWindow, PointerEventButton},
    LogicalPosition,
};
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_layer, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, Layer, LayerShell, LayerShellHandler, LayerSurface, LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};
pub struct Bar {
    config: BarConfig,
    pointer: Option<wl_pointer::WlPointer>,
    window: Rc<MinimalSoftwareWindow>,
    pub software_buffer: Vec<RgbaPixel>,
    pool: SlotPool,
    registry_state: RegistryState,
    seat_state: SeatState,
    layer_name: String,
    shm: Shm,
    output_state: OutputState,
    instances: Vec<BarInstance>,
    compositor: CompositorState,
    pub exit: bool,
    layer_shell: LayerShell,
}
impl Bar {
    pub fn new(
        window: Rc<MinimalSoftwareWindow>,
        start_pixel: RgbaPixel,
        position: Anchor,
        layer_name: &str,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, EventQueue)> {
        let conn = Connection::connect_to_env()?;
        let (config, event_queue) = BarConfig::new(&conn, position, width, height)?;
        let shm = Shm::bind(&config.globals, &config.qh).expect("wl_shm is not available");
        let pool = SlotPool::new((config.width * config.height * 4) as usize, &shm)?;
        let layer_shell = LayerShell::bind(&config.globals, &config.qh)?;
        let compositor = CompositorState::bind(&config.globals, &config.qh)?;

        Ok((
            Self {
                pool,
                registry_state: RegistryState::new(&config.globals),
                seat_state: SeatState::new(&config.globals, &config.qh),
                output_state: OutputState::new(&config.globals, &config.qh),
                config,
                shm,
                compositor,
                layer_name: String::from(layer_name),
                layer_shell,
                window,
                software_buffer: vec![start_pixel; (width * height) as usize],
                exit: false,
                pointer: None,
                instances: Vec::new(),
            },
            event_queue,
        ))
    }
    fn draw(&mut self) -> anyhow::Result<()> {
        let width = self.config.width;
        let height = self.config.height;
        let stride = width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");
        for (r, p) in canvas.iter_mut().zip(
            self.software_buffer
                .iter()
                .flat_map(|p| [p.blue, p.green, p.red, p.alpha]),
        ) {
            *r = p;
        }

        // Damage the entire window
        for instance in self.instances.iter_mut().filter(|i| i.configured) {
            instance
                .layer
                .wl_surface()
                .damage_buffer(0, 0, width as i32, height as i32);

            // Request our next frame
            instance
                .layer
                .wl_surface()
                .frame(&self.config.qh, instance.layer.wl_surface().clone());
            // Attach and commit to present.
            buffer
                .attach_to(instance.layer.wl_surface())
                .expect("buffer attach");
            instance.layer.commit();
        }
        Ok(())
    }
}

pub struct BarConfig {
    globals: GlobalList,
    position: Anchor,
    width: u32,
    height: u32,
    qh: QueueHandle<Bar>,
}
impl BarConfig {
    fn new(
        conn: &Connection,
        position: Anchor,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, EventQueue)> {
        let (globals, event_queue) = registry_queue_init(conn)?;
        let qh = event_queue.handle();
        Ok((
            Self {
                qh,
                globals,
                position,
                width,
                height,
            },
            event_queue,
        ))
    }
}

pub struct BarInstance {
    configured: bool,
    layer: LayerSurface,
    output: wl_output::WlOutput,
}

impl BarInstance {
    pub fn new(layer: LayerSurface, output: wl_output::WlOutput) -> Self {
        Self {
            configured: false,
            layer,
            output,
        }
    }
}

impl CompositorHandler for Bar {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }
    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw().unwrap();
    }
}
impl OutputHandler for Bar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let surface = self.compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Top,
            Some(&self.layer_name),
            Some(&output),
        );
        layer.set_anchor(self.config.position);
        layer.set_size(self.config.width, self.config.height);
        layer.set_exclusive_zone(self.config.height as i32);
        layer.commit();
        let instance = BarInstance::new(layer, output);
        self.instances.push(instance);
        event!(
            Level::DEBUG,
            "output created. {} outputs exist",
            self.instances.len()
        );
    }
    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        self.instances.retain(|i| i.output != output);
        event!(
            Level::DEBUG,
            "output destroyed. {} outputs remain",
            self.instances.len()
        );
    }
}
impl ShmHandler for Bar {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}
impl SeatHandler for Bar {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _seat: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer if self.pointer.is_none() => {
                event!(Level::DEBUG, "Set pointer capability");
                let pointer = self
                    .seat_state
                    .get_pointer(qh, &seat)
                    .expect("Failed to create pointer");
                self.pointer = Some(pointer);
            }
            _ => {}
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer if self.pointer.is_some() => {
                event!(Level::DEBUG, "Unset pointer capability");
                self.pointer.take().unwrap().release();
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}
impl PointerHandler for Bar {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use slint::platform::WindowEvent;
        use PointerEventKind::*;
        for event in events {
            if self
                .instances
                .iter()
                .all(|i| event.surface != *i.layer.wl_surface())
            {
                continue;
            }
            let position = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                Enter { .. } => {}
                Leave { .. } => {}
                Motion { .. } => {
                    self.window
                        .dispatch_event(WindowEvent::PointerMoved { position });
                }
                Press {
                    button: button_id, ..
                } => {
                    if let Some(button) = parse_button_id(button_id) {
                        self.window
                            .dispatch_event(WindowEvent::PointerPressed { position, button })
                    }
                }
                Release {
                    button: button_id, ..
                } => {
                    if let Some(button) = parse_button_id(button_id) {
                        self.window
                            .dispatch_event(WindowEvent::PointerReleased { position, button })
                    }
                }
                Axis { .. } => {}
            }
        }
    }
}
impl LayerShellHandler for Bar {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}
    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        _configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let mut instance = self
            .instances
            .iter_mut()
            .find(|i| i.layer == *layer)
            .expect("unable to configure layer.  It doesn't exist");
        if !instance.configured {
            instance.configured = true;
            self.draw().unwrap();
        }
    }
}

delegate_compositor!(Bar);
delegate_output!(Bar);
delegate_seat!(Bar);
delegate_shm!(Bar);
delegate_pointer!(Bar);
delegate_registry!(Bar);
delegate_layer!(Bar);

impl ProvidesRegistryState for Bar {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}

fn parse_button_id(id: u32) -> Option<PointerEventButton> {
    match id {
        272 => Some(PointerEventButton::Left),
        273 => Some(PointerEventButton::Right),
        274 => Some(PointerEventButton::Middle),
        _ => None,
    }
}
