use std::rc::Rc;

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
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    protocol::{wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, EventQueue, QueueHandle,
};
type SlintTarget = Rc<MinimalSoftwareWindow>;
use crate::ui::RgbaPixel;
pub struct Bar {
    config: BarConfig,
    pointer: Option<wl_pointer::WlPointer>,
    window: Rc<MinimalSoftwareWindow>,
    pub software_buffer: Vec<RgbaPixel>,
    pool: SlotPool,
    registry_state: RegistryState,
    seat_state: SeatState,
    shm: Shm,
    output_state: OutputState,
    instances: Vec<BarInstance>,
}
impl Bar {
    pub fn new(
        conn: &Connection,
        window: Rc<MinimalSoftwareWindow>,
        start_pixel: RgbaPixel,
        position: Anchor,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, EventQueue<Self>)> {
        let (config, event_queue) = BarConfig::new(conn, position, width, height)?;
        let shm = Shm::bind(&config.globals, &config.qh).expect("wl_shm is not available");
        let pool = SlotPool::new((config.width * config.height * 4) as usize, &shm)?;

        (
            Self {
                pool,
                registry_state: RegistryState::new(&config.globals),
                seat_state: SeatState::new(&config.globals, &config.qh),
                output_state: OutputState::new(&config.globals, &config.qh),
                config,
                shm,
                window,
                software_buffer: vec![start_pixel; (width * height) as usize],
                pointer: None,
                instances: Vec::new(),
            },
            event_queue,
        );
        todo!()
    }
}

pub struct BarConfig {
    exit: bool,
    globals: GlobalList,
    position: Anchor,
    width: u32,
    height: u32,
    // protocols: Protocols,
    qh: QueueHandle<Bar>,
}
impl BarConfig {
    fn new(
        conn: &Connection,
        position: Anchor,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, EventQueue<Bar>)> {
        let (globals, event_queue) = registry_queue_init(conn)?;
        let qh = event_queue.handle();
        Ok((
            Self {
                // protocols: Protocols::new(&globals, &qh)?,
                qh,
                exit: false,
                globals,
                position,
                width,
                height,
            },
            event_queue,
        ))
    }
}
// pub struct Protocols {
//     layer_shell: LayerShell,
// }
// impl Protocols {
//     fn new(globals: &GlobalList, qh: &QueueHandle<Bar>) -> anyhow::Result<Self> {
//         Ok(Self {
//             layer_shell: LayerShell::bind(globals, qh)?,
//         })
//     }
// }

pub struct BarInstance {
    configured: bool,
    closed: bool,
    compositor: CompositorState,
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
        conn: &Connection,
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        time: u32,
    ) {
    }
}
impl OutputHandler for Bar {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        println!("output created");
        let compositor = CompositorState::bind(&self.config.globals, qh)
            .expect("wl_compositor is not available");
        let surface = compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Top,
            Some("simple_layer"),
            None,
        );
        // Configure the layer surface, providing things like the anchor on screen, desired size and the keyboard
        // interactivity
        layer.set_anchor(self.position);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(self.width, self.height);
        layer.set_exclusive_zone(self.height as i32);

        // In order for the layer surface to be mapped, we need to perform an initial commit with no attached\
        // buffer. For more info, see WaylandSurface::commit
        //
        // The compositor will respond with an initial configure that we can then use to present to the layer
        // surface with the correct options.
        layer.commit();
        self.layer = layer;
        self.first_configure = true;
    }
    fn update_output(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
    }
    fn output_destroyed(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
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
    fn new_seat(&mut self, conn: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat) {}
    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer if self.pointer.is_none() => {
                println!("Set pointer capability");
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
                println!("Unset pointer capability");
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
            // // Ignore events for other surfaces DOTHIS LATER
            // if &event.surface != self.layer.wl_surface() {
            //     continue;
            // }
            eprintln!("pointerhandler ignore surfaces");
            let position = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                Enter { .. } => {
                    println!("Pointer entered @{:?}", event.position);
                }
                Leave { .. } => {
                    println!("Pointer left");
                }
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
                Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}
impl LayerShellHandler for BarInstance {
    fn closed(&mut self, conn: &Connection, qh: &QueueHandle<Self>, layer: &LayerSurface) {
        self.closed = true;
    }
    fn configure(
        &mut self,
        conn: &Connection,
        qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        serial: u32,
    ) {
        self.configured = true;
    }
}

pub struct BarLayer {
    pub exit: bool,
    first_configure: bool,
    globals: GlobalList,
    height: u32,
    layer: LayerSurface,
    layer_shell: LayerShell,
    output_state: OutputState,
    pointer: Option<wl_pointer::WlPointer>,
    pool: SlotPool,
    registry_state: RegistryState,
    seat_state: SeatState,
    shm: Shm,
    pub software_buffer: Vec<RgbaPixel>,
    width: u32,
    window: Rc<MinimalSoftwareWindow>,
    position: Anchor,
}
impl BarLayer {
    pub fn new(
        conn: &Connection,
        window: Rc<MinimalSoftwareWindow>,
        position: Anchor,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Self, EventQueue<Self>)> {
        // Enumerate the list of globals to get the protocols the server implements.
        let (globals, event_queue) = registry_queue_init(conn)?;
        let qh = event_queue.handle();

        // The compositor (not to be confused with the server which is commonly called the compositor) allows
        // configuring surfaces to be presented.
        let compositor =
            CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");
        // This app uses the wlr layer shell, which may not be available with every compositor.
        let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");
        // Since we are not using the GPU in this example, we use wl_shm to allow software rendering to a buffer
        // we share with the compositor process.
        let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

        // A layer surface is created from a surface.
        let surface = compositor.create_surface(&qh);

        // And then we create the layer shell.
        let layer =
            layer_shell.create_layer_surface(&qh, surface, Layer::Top, Some("simple_layer"), None);
        // Configure the layer surface, providing things like the anchor on screen, desired size and the keyboard
        // interactivity
        layer.set_anchor(position);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(width, height);
        layer.set_exclusive_zone(height as i32);

        // In order for the layer surface to be mapped, we need to perform an initial commit with no attached\
        // buffer. For more info, see WaylandSurface::commit
        //
        // The compositor will respond with an initial configure that we can then use to present to the layer
        // surface with the correct options.
        layer.commit();

        // We don't know how large the window will be yet, so lets assume the minimum size we suggested for the
        // initial memory allocation.
        let pool =
            SlotPool::new((width * height * 4) as usize, &shm).expect("Failed to create pool");
        Ok((
            BarLayer {
                // Seats and outputs may be hotplugged at runtime, therefore we need to setup a registry state to
                // listen for seats and outputs.
                registry_state: RegistryState::new(&globals),
                window,
                seat_state: SeatState::new(&globals, &qh),
                output_state: OutputState::new(&globals, &qh),
                shm,
                globals,
                software_buffer: vec![RgbaPixel::transparent(); (width * height) as usize],
                exit: false,
                position,
                first_configure: true,
                pool,
                width,
                layer_shell,
                height,
                layer,
                pointer: None,
            },
            event_queue,
        ))

        // We don't draw immediately, the configure will notify us when to first draw.
    }
}

impl CompositorHandler for BarLayer {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
        // Not needed for this example.
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        self.draw(qh);
    }
}

impl OutputHandler for BarLayer {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
        println!("output created");
        let compositor =
            CompositorState::bind(&self.globals, qh).expect("wl_compositor is not available");
        let surface = compositor.create_surface(qh);
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Top,
            Some("simple_layer"),
            None,
        );
        // Configure the layer surface, providing things like the anchor on screen, desired size and the keyboard
        // interactivity
        layer.set_anchor(self.position);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(self.width, self.height);
        layer.set_exclusive_zone(self.height as i32);

        // In order for the layer surface to be mapped, we need to perform an initial commit with no attached\
        // buffer. For more info, see WaylandSurface::commit
        //
        // The compositor will respond with an initial configure that we can then use to present to the layer
        // surface with the correct options.
        layer.commit();
        self.layer = layer;
        self.first_configure = true;
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
        _output: wl_output::WlOutput,
    ) {
        println!("output died");
    }
}

impl LayerShellHandler for BarLayer {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {}

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
            self.width = 256;
            self.height = 256;
        } else {
            self.width = configure.new_size.0;
            self.height = configure.new_size.1;
        }

        // Initiate the first draw.
        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl SeatHandler for BarLayer {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        match capability {
            Capability::Pointer if self.pointer.is_none() => {
                println!("Set pointer capability");
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
                println!("Unset pointer capability");
                self.pointer.take().unwrap().release();
            }
            _ => {}
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl PointerHandler for BarLayer {
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
            // Ignore events for other surfaces
            if &event.surface != self.layer.wl_surface() {
                continue;
            }
            let position = LogicalPosition::new(event.position.0 as f32, event.position.1 as f32);
            match event.kind {
                Enter { .. } => {
                    println!("Pointer entered @{:?}", event.position);
                }
                Leave { .. } => {
                    println!("Pointer left");
                }
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
                Axis {
                    horizontal,
                    vertical,
                    ..
                } => {
                    println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                }
            }
        }
    }
}

impl ShmHandler for BarLayer {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl BarLayer {
    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = self.width as i32 * 4;

        let (buffer, canvas) = self
            .pool
            .create_buffer(
                width as i32,
                height as i32,
                stride,
                wl_shm::Format::Argb8888,
            )
            .expect("create buffer");
        //        println!("{:?}", self.software_buffer[40]);
        for (r, p) in canvas.iter_mut().zip(
            self.software_buffer
                .iter()
                .flat_map(|p| [p.blue, p.green, p.red, p.alpha]),
        ) {
            *r = p;
        }

        // Draw to the window:

        // Damage the entire window
        self.layer
            .wl_surface()
            .damage_buffer(0, 0, width as i32, height as i32);

        // Request our next frame
        self.layer
            .wl_surface()
            .frame(qh, self.layer.wl_surface().clone());

        // Attach and commit to present.
        buffer
            .attach_to(self.layer.wl_surface())
            .expect("buffer attach");
        self.layer.commit();

        // TODO save and reuse buffer when the window size is unchanged.  This is especially
        // useful if you do damage tracking, since you don't need to redraw the undamaged parts
        // of the canvas.
    }
}

delegate_compositor!(BarLayer);
delegate_output!(BarLayer);

delegate_compositor!(Bar);
delegate_output!(Bar);
delegate_seat!(Bar);
delegate_shm!(Bar);
delegate_pointer!(Bar);
delegate_registry!(Bar);
delegate_layer!(Bar);
delegate_layer!(BarInstance);

delegate_shm!(BarLayer);

delegate_seat!(BarLayer);
delegate_pointer!(BarLayer);

delegate_layer!(BarLayer);

delegate_registry!(BarLayer);

impl ProvidesRegistryState for BarLayer {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState];
}
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
