use crate::gui::shader;
use crate::gui::shader::PixmapShaderInterop;
use crate::pixmap::traits::PixmapRawRead;
use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::{Maintain, MaintainBase};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::window::Window;

use super::utils::async_to_sync;

pub(super) struct RenderState<P: PixmapRawRead> {
    /// The instance is the first thing created when using wgpu.
    /// Its main purpose is to create Adapters and Surfaces.
    wgpu_instance: wgpu::Instance,

    /// The surface is the part of the window that we draw to. We need it to draw directly to the screen.
    /// The window needs to implement [`HasRawWindowHandle`](raw_window_handle::HasRawWindowHandle) trait to create a surface.
    /// Fortunately, *winit*'s Window fits the bill.
    /// We also need it to request our adapter.
    surface: wgpu::Surface,

    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    pixel_texture: wgpu::Texture,

    /// The configuration which was used to create the surface.
    /// Required for resizing.
    config: wgpu::SurfaceConfiguration,

    /// The physical size of the underlying window.
    /// Required for resizing.
    size: winit::dpi::PhysicalSize<u32>,

    pixmap: Arc<P>,

    background_color: wgpu::Color,
}

impl<P: PixmapRawRead> RenderState<P> {
    pub fn new(window: &Window, pixmap: Arc<P>) -> Result<Self> {
        let size = window.inner_size();

        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = unsafe {
            wgpu_instance
                .create_surface(&window)
                .with_context(|| "Could not create rendering surface")?
        };

        // The adapter is a handle to our actual graphics card.
        // It can be uses to get information about the graphics card such as its name and what backend the adapter uses.
        // We use this to create our Device and Queue.
        let adapter = async_to_sync(wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .with_context(|| "Could not get a GPU adapter for rendering")?;

        let (device, queue) = async_to_sync(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .with_context(|| "Could not setup channel to GPU")?;

        let config = surface
            .get_default_config(&adapter, window.inner_size().width, window.inner_size().height)
            .with_context(|| "Could not configure rendering surface for the GPU")?;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PixelflutShader"),
            source: shader::get_shader_source(),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let (pixmap_width, pixmap_height) = pixmap.get_size()?;
        let pixel_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: pixmap_width as u32,
                height: pixmap_height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("Pixmap"),
            view_formats: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        Ok(Self {
            wgpu_instance,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            pixel_texture,
            pixmap,
            background_color: wgpu::Color::BLACK,
        })
    }

    /// Handle resizing of the underlying window by adjusting the render config
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Process an input event and return whether it has been fully processed.
    pub fn input(&mut self, input: &KeyboardInput) {
        if let KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(VirtualKeyCode::Space),
            ..
        } = input
        {
            if self.background_color == wgpu::Color::BLACK {
                self.background_color = wgpu::Color::WHITE;
            } else {
                self.background_color = wgpu::Color::BLACK;
            }
        }
    }

    pub fn render(&mut self) {
        debug!("========== REDRAWING PIXMAP ==========");

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // render from the gpu buffer onto the texture
        self.pixmap.write_gpu_texture(&self.queue, &self.pixel_texture);
        let mut cmd_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut render_pass = cmd_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("pixmap_render"),
                depth_stencil_attachment: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: true,
                    },
                })],
            });

            self.device.poll(Maintain::Poll);
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..4, 0..1)
        }
        self.queue.submit(Some(cmd_encoder.finish()));

        frame.present();
        //self.wgpu_instance.poll_all(true);
    }
}
