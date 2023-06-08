//! Blocking managament code for windows and gpu handles

use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::{
    include_wgsl, Adapter, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, ShaderModule, Surface,
};
use winit::dpi::LogicalSize;
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{Window, WindowBuilder},
};

use crate::gui::utils::async_to_sync;
use crate::pixmap::traits::PixmapRawRead;

use super::rendering::RenderState;

/// A data structure that holds all context information that is required for rendering
pub(super) struct GuiContext<P: PixmapRawRead> {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    render_state: RenderState,
    pixmap: Arc<P>,
}

impl<P: PixmapRawRead + 'static> GuiContext<P> {
    pub fn new(pixmap: Arc<P>) -> Result<Self> {
        log::debug!("Constructing GuiContext");

        let (pixmap_width, pixmap_height) = pixmap.get_size()?;
        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

        let window = WindowBuilder::new()
            .with_inner_size(Size::Logical(LogicalSize::new(
                pixmap_width as f64,
                pixmap_height as f64,
            )))
            .build(&event_loop)
            .with_context(|| "Could not construct window")?;

        let render_state = RenderState::new(&window)?;

        Ok(Self {
            window,
            event_loop: Some(event_loop),
            render_state,
            pixmap,
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

                    // handle resizing when the window size changes
                    Event::WindowEvent {
                        event: WindowEvent::Resized(physical_size),
                        ..
                    } => {
                        self.render_state.resize(physical_size);
                    }

                    // handle resizing when the window scaling changes
                    Event::WindowEvent {
                        event: WindowEvent::ScaleFactorChanged { new_inner_size, .. },
                        ..
                    } => {
                        self.render_state.resize(*new_inner_size);
                    }

                    // schedule a redraw when polled
                    Event::NewEvents(StartCause::Poll) => {
                        self.window.request_redraw();
                    }

                    // redraw the canvas if something determined that to be required
                    Event::RedrawRequested(_) => {
                        self.render_state.render(&self.pixmap);
                    }

                    // forward keyboard events to the rendering logic
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput { input, .. },
                        ..
                    } => {
                        self.render_state.input(&input);
                        self.window.request_redraw();
                    }

                    // ignore all other events
                    _ => {}
                }
            })
    }
}
