use alloc::sync::Arc;

use raw_window_handle::HasRawWindowHandle;

use crate::{
    constants::{INTERNAL_COLOR_ATTACHMENT_FORMAT, INTERNAL_DEPTH_ATTACHMENT_FORMAT},
    Renderer, Texture,
};

pub trait RenderTarget: Sync + Send {
    fn size(&self) -> (u32, u32);
    fn color_attachment(&self) -> &wgpu::TextureView;
    fn submit(&mut self);
    fn output_format(&self) -> wgpu::TextureFormat;
}

pub struct WindowRenderTarget {
    swap_chain: wgpu::SwapChain,
    frame: Option<wgpu::SwapChainFrame>,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
}

impl WindowRenderTarget {
    pub fn new<W: HasRawWindowHandle>(renderer: &Renderer, window: &W, width: u32, height: u32) -> Self {
        let surface = unsafe { renderer.instance.create_surface(window) };
        let format = renderer.adapter.get_swap_chain_preferred_format(&surface).unwrap();

        let swap_chain = renderer.device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                format,
                width,
                height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );
        let frame = swap_chain.get_current_frame().unwrap();

        Self {
            swap_chain,
            frame: Some(frame),
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
        // we must drop swapchainoutput first
        self.frame = None;

        self.frame = Some(self.swap_chain.get_current_frame().unwrap())
    }

    fn color_attachment(&self) -> &wgpu::TextureView {
        &self.frame.as_ref().unwrap().output.view
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
    pub fn new(renderer: &Renderer, width: u32, height: u32) -> Self {
        let color_attachment = Arc::new(Texture::new(renderer, width, height, INTERNAL_COLOR_ATTACHMENT_FORMAT));
        let depth_attachment = Arc::new(Texture::new(renderer, width, height, INTERNAL_DEPTH_ATTACHMENT_FORMAT));

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
        INTERNAL_COLOR_ATTACHMENT_FORMAT.wgpu_type()
    }
}
