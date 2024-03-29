use alloc::{borrow::ToOwned, sync::Arc, vec::Vec};

use hashbrown::HashMap;

use super::{resource::Resource, texture::Texture, Renderer, Shader, ShaderBindingType};

pub struct Material {
    pub(crate) shader: Arc<Shader>,
    pub(crate) bind_group: wgpu::BindGroup,
}

impl Material {
    pub fn new(renderer: &Renderer, texture: &Texture) -> Self {
        Self::with_device(
            &renderer.device,
            Some(&renderer.shader_transform),
            &[("texture", texture)],
            renderer.standard_shader.clone(),
        )
    }

    pub fn with_custom_shader(renderer: &Renderer, resources: &[(&str, &dyn Resource)], shader: Arc<Shader>) -> Self {
        Self::with_device(&renderer.device, Some(&renderer.shader_transform), resources, shader)
    }

    pub(crate) fn with_device(
        device: &wgpu::Device,
        transform: Option<&dyn Resource>,
        resources: &[(&str, &dyn Resource)],
        shader: Arc<Shader>,
    ) -> Self {
        let resources = resources.iter().map(|x| (x.0.to_owned(), x.1)).collect::<HashMap<_, _>>();

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
                let resource = if *binding_name == "transform" {
                    transform.unwrap().wgpu_resource()
                } else if binding.binding_type == ShaderBindingType::Sampler {
                    wgpu::BindingResource::Sampler(&sampler)
                } else {
                    let resource = resources.get(binding_name).unwrap();
                    resource.wgpu_resource()
                };

                wgpu::BindGroupEntry {
                    binding: binding.binding,
                    resource,
                }
            })
            .collect::<Vec<_>>();

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &shader.bind_group_layout,
            entries: &entries,
            label: None,
        });

        Self { shader, bind_group }
    }
}
