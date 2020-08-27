use raw_window_handle::HasRawWindowHandle;

use crate::{Renderer, Texture, TextureFormat};

pub trait RenderTarget: Sync + Send {
    fn size(&self) -> (u32, u32);
    fn color_attachment(&self) -> &wgpu::TextureView;
    fn depth_attachment(&self) -> &wgpu::TextureView;
    fn submit(&mut self);
}

pub struct WindowRenderTarget {
    swap_chain: wgpu::SwapChain,
    frame: Option<wgpu::SwapChainFrame>,
    depth_view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl WindowRenderTarget {
    pub fn new<W: HasRawWindowHandle>(renderer: &Renderer, window: &W, width: u32, height: u32) -> Self {
        let surface = unsafe { renderer.instance.create_surface(window) };

        let mut swap_chain = renderer.device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: wgpu::TextureFormat::Bgra8Unorm,
                width,
                height,
                present_mode: wgpu::PresentMode::Mailbox,
            },
        );
        let frame = swap_chain.get_current_frame().unwrap();

        let depth = Texture::new(&renderer, width, height, TextureFormat::Depth32);

        Self {
            swap_chain,
            frame: Some(frame),
            depth_view: depth.texture_view,
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

    fn depth_attachment(&self) -> &wgpu::TextureView {
        &self.depth_view
    }
}
