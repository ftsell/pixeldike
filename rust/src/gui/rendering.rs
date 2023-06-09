use crate::gui::shader;
use crate::gui::shader::{PixmapShaderInterop, Vertex, VERTEX_INDICES, VERTICES};
use crate::gui::texture::TextureState;
use crate::pixmap::traits::PixmapRawRead;
use anyhow::{Context, Result};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use wgpu::BindGroup;
use winit::event::KeyboardInput;
use winit::window::Window;

use super::utils::async_to_sync;

pub(super) struct RenderState<P>
where
    P: PixmapRawRead,
{
    /// The instance is the first thing created when using wgpu.
    /// Its main purpose is to create Adapters and Surfaces.
    #[allow(unused)]
    wgpu_instance: wgpu::Instance,

    /// The surface is the part of the window that we draw to. We need it to draw directly to the screen.
    /// The window needs to implement [`HasRawWindowHandle`](raw_window_handle::HasRawWindowHandle) trait to create a surface.
    /// Fortunately, *winit*'s Window fits the bill.
    /// We also need it to request our adapter.
    surface: wgpu::Surface,

    device: wgpu::Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,

    /// The configuration which was used to create the surface.
    /// Required for resizing.
    config: wgpu::SurfaceConfiguration,

    /// The physical size of the underlying window.
    /// Required for resizing.
    size: winit::dpi::PhysicalSize<u32>,

    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,

    texture: TextureState,
    texture_bind_group: BindGroup,

    pixmap: Arc<P>,
}

impl<P> RenderState<P>
where
    P: PixmapRawRead,
{
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

        let surface_config = surface
            .get_default_config(&adapter, window.inner_size().width, window.inner_size().height)
            .with_context(|| "Could not configure rendering surface for the GPU")?;
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PixelflutShader"),
            source: shader::get_shader_source(),
        });

        let texture_state = TextureState::new(&device, pixmap.as_ref()).unwrap();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("pixmap_texture_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("pixmap_texture_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_state.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_state.sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pixmap_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
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
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(VERTEX_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            wgpu_instance,
            surface,
            device,
            queue,
            config: surface_config,
            size,
            render_pipeline,
            texture: texture_state,
            vertex_buffer,
            index_buffer,
            texture_bind_group: bind_group,
            pixmap,
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
    pub fn input(&mut self, _input: &KeyboardInput) {
        // TODO Implement showing / hiding of GUI
    }

    pub fn render(&mut self) {
        debug!("========== REDRAWING PIXMAP ==========");

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // load the pixmap data into the texture
        self.pixmap
            .write_to_texture(&mut self.queue, &self.texture.texture);

        // encode a single render pass which renders the pixmap state onto the surface
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                        store: true,
                    },
                })],
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..VERTEX_INDICES.len() as u32, 0, 0..1)
        }

        // execute the render pass and present the finished frame
        self.queue.submit(Some(cmd_encoder.finish()));
        frame.present();
    }
}
