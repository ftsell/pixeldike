//! Data structures that are shared with the shader code

use std::sync::Arc;
use crate::pixmap::traits::PixmapRawRead;
use wgpu::{Buffer, MapMode, ShaderSource, VertexBufferLayout};

/// Interoperability trait to make pixmaps usable for gpu shaders
pub(super) trait PixmapShaderInterop {
    /// Fill the given gpu vertex buffer with pixel data in [`Vertex`] format
    fn fill_vertex_buffer(&self, vertex_buffer: Arc<Buffer>);
}

impl<T: PixmapRawRead> PixmapShaderInterop for T {
    fn fill_vertex_buffer(&self, vertex_buffer: Arc<Buffer>) {
        vertex_buffer.slice(..).map_async(MapMode::Write, move |map_result| {
            map_result.unwrap();
            let view = vertex_buffer.slice(..).get_mapped_range_mut()
            log::debug!("{}", view.len());
        });
    }
}

/// Return the content of `shader.wgsl` in a usable format
pub(super) fn get_shader_source() -> ShaderSource<'static> {
    ShaderSource::Wgsl(include_str!("shader.wgsl").into())
}

/// The data that is stored for each vertex and which determines how it is drawn.
///
/// This basically represents one pixel on the pixmap.
///
/// It should match the `VertexInput` struct in `shader.wgsl`.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(super) struct Vertex {
    /// The x, y, and z of the vertex in 2d space
    position: [f32; 2],
    /// red, green, and blue values for the vertex
    color: [f32; 3],
}

impl Vertex {
    /// Create a [`VertexBufferLayout`] corresponding to this vertex for use in a [`RenderPipeline`](wgpu::RenderPipeline)
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}
