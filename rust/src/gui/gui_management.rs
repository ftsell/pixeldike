//! Blocking managament code for windows and gpu handles

use anyhow::{Context, Result};
use wgpu::{
    include_wgsl, Adapter, Device, DeviceDescriptor, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, ShaderModule, Surface, Instance,
};
use winit::{
    dpi::{PhysicalSize, Size},
    event::{Event, StartCause, WindowEvent},
    event_loop::{EventLoop, EventLoopBuilder},
    platform::unix::EventLoopBuilderExtUnix,
    window::{Window, WindowBuilder},
};

use crate::gui::utils::async_to_sync;

use super::rendering::RenderState;

/// A data structure that holds all context information that is required for rendering
pub(super) struct GuiContext {
    window: Window,
    event_loop: Option<EventLoop<()>>,
    render_state: RenderState,
}

impl GuiContext {
    pub fn new() -> Result<Self> {
        log::debug!("Constructing GuiContext");

        let event_loop = EventLoopBuilder::new().with_any_thread(true).build();

        let window = WindowBuilder::new()
            .with_inner_size(Size::Physical(PhysicalSize::new(800, 600)))
            .build(&event_loop)
            .with_context(|| "Could not construct window")?;

        let render_state = RenderState::new(&window)?;

        Ok(Self {
            window,
            event_loop: Some(event_loop),
            render_state,
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
                        //self.window.request_redraw();
                    }

                    // redraw the canvas if something determined that to be required
                    Event::RedrawRequested(_) => {
                        self.render_state.render();
                    }

                    // ignore all other events
                    _ => {}
                }
            })
    }
}
