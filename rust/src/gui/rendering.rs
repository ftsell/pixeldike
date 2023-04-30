use anyhow::{Context, Result};


use super::utils::async_to_sync;


pub(super) struct RenderState {
    wgpu_instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    //size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
}

impl RenderState {
    pub fn new<W>(window: &W) -> Result<Self>
    where
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    {
        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        let surface = unsafe {
            wgpu_instance
                .create_surface(&window)
                .with_context(|| "Could not create rendering surface")?
        };

        let adapter = async_to_sync(wgpu_instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .with_context(|| "Could not get a GPU adapter for rendering")?;

        let (device, queue) = async_to_sync(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
            .with_context(|| "Could not setup channes to GPU")?;

        let config = surface
            .get_default_config(&adapter, 800, 600)
            .with_context(|| "Could not configure rendering surface for the GPU")?;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("PixelflutShader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
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
            //size,
            render_pipeline,
        })
    }

    pub fn render(&mut self) {
        debug!("========== REDRAWING PIXMAP ==========");

        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        // render from the gpu buffer onto the texture
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
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..3, 0..1)
        }
        self.queue.submit(Some(cmd_encoder.finish()));

        frame.present();
        //self.wgpu_instance.poll_all(true);
    }
}
