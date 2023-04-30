//! Blocking managament code for windows and gpu handles

use anyhow::{Context, Result};
use wgpu::{
    include_wgsl, Adapter, Device, DeviceDescriptor, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, ShaderModule, Surface, TextureViewDescriptor, CommandEncoderDescriptor, RenderPassDescriptor, RenderPassColorAttachment, Operations, LoadOp, Color, Instance,
};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{Window, WindowBuilder},
};

use crate::gui::utils::async_to_sync;

/// A data structure that holds all context information that is required for rendering
pub(super) struct GuiContext {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    wgpu_instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    shader: ShaderModule,
}

impl GuiContext {
    pub fn new() -> Result<Self> {
        log::debug!("Constructing GuiContext");

        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

        let window = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize::new(800, 600)))
            .build(&event_loop)
            .with_context(|| "Could not construct window")?;

        let wgpu_instance = wgpu::Instance::new(InstanceDescriptor::default());

        let surface = unsafe {
            wgpu_instance
                .create_surface(&window)
                .with_context(|| "Could not create rendering surface")?
        };

        let adapter = async_to_sync(wgpu_instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .with_context(|| "Could not get a GPU adapter for rendering")?;

        let (device, queue) = async_to_sync(adapter.request_device(&DeviceDescriptor::default(), None))
            .with_context(|| "Could not setup channes to GPU")?;

        surface.configure(
            &device,
            &surface
                .get_default_config(&adapter, 800, 600)
                .with_context(|| "Could not configure rendering surface for the GPU")?,
        );

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        Ok(Self {
            window,
            event_loop: Some(event_loop),
            wgpu_instance,
            surface,
            adapter,
            device,
            queue,
            shader,
        })
    }

    pub fn run(mut self) {
        self.event_loop
            .take()
            .unwrap()
            .run(move |event, _, control_flow| {
                // Set the event loop to run even if the os hasn't dispatched any events
                // we require this because pixmap updates happen all the time, independent of os events
                control_flow.set_poll();

                match event {
                    // close the window if the X has been clicked
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        control_flow.set_exit();
                    }

                    // schedule a redraw when polled
                    Event::NewEvents(StartCause::Poll) => {
                        self.window.request_redraw();
                    }

                    // redraw the canvas if something determined that to be required
                    Event::RedrawRequested(_) => {
                        self.redraw();
                    }

                    // ignore all other events
                    _ => {}
                }
            })
    }

    fn redraw(&mut self) {
        debug!("========== REDRAWING PIXMAP ==========");

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        // render from the gpu buffer onto the texture
        let mut cmd_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor::default());
        {
            let render_pass = cmd_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("pixmap_render"),
                depth_stencil_attachment: None,
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(wgpu::Color::RED),
                        store: true,
                    },
                })],
            });
        }
        self.queue.submit(Some(cmd_encoder.finish()));

        frame.present();
        self.wgpu_instance.poll_all(true);
    }
}
