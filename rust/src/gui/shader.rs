//! Data structures that are shared with the shader code

use crate::pixmap;
use crate::pixmap::traits::PixmapRawRead;
use std::num::NonZeroU32;
use wgpu::{Queue, ShaderSource, Texture, VertexBufferLayout};

/// Interoperability trait to make pixmaps usable for gpu shaders
pub(super) trait PixmapShaderInterop {
    fn write_gpu_texture(&self, queue: &Queue, texture: &Texture);
}

impl<T: PixmapRawRead> PixmapShaderInterop for T {
    fn write_gpu_texture(&self, queue: &Queue, texture: &Texture) {
        let texture_data: Vec<u8> = self
            .get_raw_data()
            .unwrap()
            .iter()
            .flat_map(|&c| <pixmap::Color as Into<[u8; 4]>>::into(c))
            .collect();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &texture_data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * texture.size().width),
                rows_per_image: NonZeroU32::new(texture.size().height),
            },
            texture.size(),
        );
    }
}

/// Return the content of `shader.wgsl` in a usable format
pub(super) fn get_shader_source() -> ShaderSource<'static> {
    ShaderSource::Wgsl(include_str!("shader.wgsl").into())
}
