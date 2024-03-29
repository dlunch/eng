use alloc::{vec, vec::Vec};

use super::{resource::Resource, Renderer};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TextureFormat {
    Rgba8Unorm,
    Bgra8Unorm,
    Rgba16Float,
    Depth32,
}

impl TextureFormat {
    pub(crate) fn wgpu_format(&self) -> wgpu::TextureFormat {
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
        Self::with_device(&renderer.device, width, height, format)
    }

    pub(crate) fn with_device(device: &wgpu::Device, width: u32, height: u32, format: TextureFormat) -> Self {
        let texture = Self::create(device, width, height, format);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture_view }
    }

    pub fn with_texels(renderer: &Renderer, width: u32, height: u32, texels: &[u8], format: TextureFormat) -> Self {
        Self::with_device_texels(&renderer.device, &renderer.queue, width, height, texels, format)
    }

    pub(crate) fn with_device_texels(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        texels: &[u8],
        format: TextureFormat,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        if format == TextureFormat::Bgra8Unorm {
            // webgl doesn't support bgra texture
            let mut rgba_texels = Vec::with_capacity(texels.len());
            for i in 0..texels.len() / 4 {
                rgba_texels.push(texels[i * 4 + 2]);
                rgba_texels.push(texels[i * 4 + 1]);
                rgba_texels.push(texels[i * 4]);
                rgba_texels.push(texels[i * 4 + 3]);
            }
            return Self::with_texels(renderer, width, height, &rgba_texels, TextureFormat::Rgba8Unorm);
        }

        let texture = Self::create(device, width, height, format);

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: core::num::NonZeroU32::new(format.bytes_per_row() as u32 * width),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Self { texture_view }
    }

    pub fn with_compressed_texels(renderer: &Renderer, width: u32, height: u32, data: &[u8], format: CompressedTextureFormat) -> Self {
        let uncompressed = Self::decode_texture(data, width, height, &format);

        Self::with_texels(renderer, width, height, &uncompressed, format.decoded_format())
    }

    fn create(device: &wgpu::Device, width: u32, height: u32, format: TextureFormat) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: format.wgpu_format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: None,
        })
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

impl Resource for Texture {
    fn wgpu_resource(&self) -> wgpu::BindingResource {
        wgpu::BindingResource::TextureView(&self.texture_view)
    }
}
