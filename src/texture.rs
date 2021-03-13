use alloc::{vec, vec::Vec};

use crate::Renderer;

pub enum TextureFormat {
    Rgba8Unorm,
    Bgra8Unorm,
    Rgba16Float,
    Depth32,
}

impl TextureFormat {
    pub(crate) fn wgpu_type(&self) -> wgpu::TextureFormat {
        match self {
            TextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
            TextureFormat::Bgra8Unorm => wgpu::TextureFormat::Bgra8Unorm,
            TextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
            TextureFormat::Depth32 => wgpu::TextureFormat::Depth32Float,
        }
    }
    pub(crate) fn bytes_per_row(&self) -> usize {
        match self {
            TextureFormat::Rgba8Unorm => 4,
            TextureFormat::Bgra8Unorm => 4,
            TextureFormat::Rgba16Float => 8,
            TextureFormat::Depth32 => 4,
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub enum CompressedTextureFormat {
    BC1,
    BC2,
    BC3,
}

impl CompressedTextureFormat {
    pub(crate) fn decoded_format(&self) -> TextureFormat {
        match self {
            CompressedTextureFormat::BC1 => TextureFormat::Rgba8Unorm,
            CompressedTextureFormat::BC2 => TextureFormat::Rgba8Unorm,
            CompressedTextureFormat::BC3 => TextureFormat::Rgba8Unorm,
        }
    }
}

pub struct Texture {
    pub(crate) texture_view: wgpu::TextureView,
}

impl Texture {
    pub fn new(renderer: &Renderer, width: u32, height: u32, format: TextureFormat) -> Self {
        let extent = wgpu::Extent3d { width, height, depth: 1 };
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.wgpu_type(),
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: None,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture_view }
    }

    pub async fn with_texels(renderer: &Renderer, width: u32, height: u32, texels: &[u8], format: TextureFormat) -> Self {
        let extent = wgpu::Extent3d { width, height, depth: 1 };
        let texture = renderer.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.wgpu_type(),
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST | wgpu::TextureUsage::RENDER_ATTACHMENT,
            label: None,
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        renderer.queue.write_texture(
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &texels,
            wgpu::TextureDataLayout {
                offset: 0,
                bytes_per_row: format.bytes_per_row() as u32 * extent.width as u32,
                rows_per_image: 0,
            },
            extent,
        );

        Self { texture_view }
    }

    pub async fn with_compressed_texels(renderer: &Renderer, width: u32, height: u32, data: &[u8], format: CompressedTextureFormat) -> Self {
        let uncompressed = Self::decode_texture(data, width, height, &format);

        Self::with_texels(renderer, width, height, &uncompressed, format.decoded_format()).await
    }

    fn decode_texture(data: &[u8], width: u32, height: u32, format: &CompressedTextureFormat) -> Vec<u8> {
        let result_size = (width as usize) * (height as usize) * 4; // RGBA
        let mut result = vec![0; result_size];

        let format = match format {
            CompressedTextureFormat::BC1 => squish::Format::Bc1,
            CompressedTextureFormat::BC2 => squish::Format::Bc2,
            CompressedTextureFormat::BC3 => squish::Format::Bc3,
        };
        format.decompress(data, width as usize, height as usize, result.as_mut());

        result
    }
}
