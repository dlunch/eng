use alloc::{sync::Arc, vec::Vec};

use hashbrown::HashMap;

use crate::{buffer::Buffer, Renderer, Shader, ShaderBindingType, Texture};

pub struct Material {
    pub(crate) shader: Arc<Shader>,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
    pub(crate) bind_group: wgpu::BindGroup,

    _textures: HashMap<&'static str, Arc<Texture>>,
    _uniforms: HashMap<&'static str, Arc<Buffer>>,
}

impl Material {
    pub fn new(
        renderer: &Renderer,
        textures: HashMap<&'static str, Arc<Texture>>,
        uniforms: HashMap<&'static str, Arc<Buffer>>,
        shader: Arc<Shader>,
    ) -> Self {
        Self::with_device(&*renderer.device, Some(&renderer.mvp_buf), textures, uniforms, shader)
    }

    pub fn with_device(
        device: &wgpu::Device,
        mvp_buf: Option<&Buffer>,
        textures: HashMap<&'static str, Arc<Texture>>,
        uniforms: HashMap<&'static str, Arc<Buffer>>,
        shader: Arc<Shader>,
    ) -> Self {
        let bindings = shader.wgpu_bindings().collect::<Vec<_>>();

        // TODO split bind groups by stage..
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &bindings,
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        // TODO wip
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            label: None,
            anisotropy_clamp: None,
            compare: None,
            border_color: None,
        });

        let entries = shader
            .bindings
            .iter()
            .map(|(binding_name, binding)| {
                let resource = match binding.binding_type {
                    ShaderBindingType::UniformBuffer => {
                        if *binding_name == "Mvp" {
                            mvp_buf.unwrap().binding_resource()
                        } else {
                            let buffer = uniforms.get(binding_name);
                            match buffer {
                                Some(x) => x.binding_resource(),
                                None => panic!("No such buffer named {}", binding_name),
                            }
                        }
                    }
                    ShaderBindingType::Texture2D => {
                        let texture = textures.get(binding_name);
                        match texture {
                            Some(x) => wgpu::BindingResource::TextureView(&x.texture_view),
                            None => panic!("No such texture named {}", binding_name),
                        }
                    }
                    ShaderBindingType::Sampler => wgpu::BindingResource::Sampler(&sampler),
                };

                wgpu::BindGroupEntry {
                    binding: binding.binding,
                    resource,
                }
            })
            .collect::<Vec<_>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &entries,
            label: None,
        });

        Self {
            shader,
            pipeline_layout,
            bind_group,
            _textures: textures,
            _uniforms: uniforms,
        }
    }
}
