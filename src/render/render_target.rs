use alloc::sync::Arc;

use super::{
    constants::{INTERNAL_COLOR_ATTACHMENT_FORMAT, INTERNAL_DEPTH_ATTACHMENT_FORMAT},
    Texture,
};

pub trait RenderTarget: Sync + Send {
    fn size(&self) -> (u32, u32);
    fn color_attachment(&self) -> &wgpu::TextureView;
    fn submit(&mut self);
    fn output_format(&self) -> wgpu::TextureFormat;
}

pub struct WindowRenderTarget {
    texture_view: Option<wgpu::TextureView>,
    frame: Option<wgpu::SurfaceTexture>,
    surface: wgpu::Surface,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl WindowRenderTarget {
    pub(crate) fn new(surface: wgpu::Surface, adapter: &wgpu::Adapter, device: &wgpu::Device, width: u32, height: u32) -> Self {
        let format = surface.get_supported_formats(adapter)[0];

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
        };

        surface.configure(device, &config);

        let frame = surface.get_current_texture().unwrap();
        let texture_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            surface,
            frame: Some(frame),
            texture_view: Some(texture_view),
            format,
            width,
            height,
        }
    }
}

impl RenderTarget for WindowRenderTarget {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn submit(&mut self) {
        self.texture_view = None;
        self.frame.take().unwrap().present();

        self.frame = Some(self.surface.get_current_texture().unwrap());
        self.texture_view = Some(self.frame.as_ref().unwrap().texture.create_view(&wgpu::TextureViewDescriptor::default()));
    }

    fn color_attachment(&self) -> &wgpu::TextureView {
        self.texture_view.as_ref().unwrap()
    }

    fn output_format(&self) -> wgpu::TextureFormat {
        self.format
    }
}

pub struct OffscreenRenderTarget {
    width: u32,
    height: u32,

    pub(crate) color_attachment: Arc<Texture>,
    pub(crate) depth_attachment: Arc<Texture>,
}

impl OffscreenRenderTarget {
    pub(crate) fn with_device(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let color_attachment = Arc::new(Texture::with_device(device, width, height, INTERNAL_COLOR_ATTACHMENT_FORMAT));
        let depth_attachment = Arc::new(Texture::with_device(device, width, height, INTERNAL_DEPTH_ATTACHMENT_FORMAT));

        Self {
            width,
            height,
            color_attachment,
            depth_attachment,
        }
    }
}

impl RenderTarget for OffscreenRenderTarget {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn submit(&mut self) {}

    fn color_attachment(&self) -> &wgpu::TextureView {
        &self.color_attachment.texture_view
    }

    fn output_format(&self) -> wgpu::TextureFormat {
        INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_format()
    }
}
