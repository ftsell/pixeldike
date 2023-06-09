//! Data structures that are shared with the shader code

use crate::pixmap::traits::PixmapRawRead;

use std::num::NonZeroU32;

/// Interoperability trait to make pixmaps usable for gpu shaders
pub(super) trait PixmapShaderInterop {
    fn write_to_texture(&self, queue: &mut wgpu::Queue, texture: &wgpu::Texture);
}

impl<P: PixmapRawRead> PixmapShaderInterop for P {
    fn write_to_texture(&self, queue: &mut wgpu::Queue, texture: &wgpu::Texture) {
        let pixel_data: Vec<u8> = self
            .get_raw_data()
            .expect("could not read pixmap data")
            .iter()
            // encode as BGRA since that is what the shader renders
            .flat_map(|&color| [color.2, color.1, color.0, 0])
            .collect();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixel_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(800 * 4),
                rows_per_image: NonZeroU32::new(600),
            },
            texture.size(),
        );
    }
}

/// Return the content of `shader.wgsl` in a usable format
pub(super) fn get_shader_source() -> wgpu::ShaderSource<'static> {
    wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
}

/// The data that is stored for each vertex and which determines how it is drawn.
///
/// It should match the `VertexInput` struct in `shader.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

impl Vertex {
    pub(super) fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

/// A list of vertices which fully fill the screen
pub(super) const VERTICES: &[Vertex] = &[
    // top-right
    Vertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    // top-left
    Vertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    // bottom-left
    Vertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    // bottom-right
    Vertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
];

pub(super) const VERTEX_INDICES: &[u16] = &[
    0, 1, 2, // top-left triangle
    0, 2, 3, // bottom-right triangle
];
