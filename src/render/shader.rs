use alloc::{string::String, vec::Vec};

use hashbrown::HashMap;

use super::Renderer;

#[derive(Clone, Eq, PartialEq)]
pub enum ShaderBindingType {
    DynamicUniformBuffer,
    UniformBuffer,
    Texture2D,
    Sampler,
}

impl ShaderBindingType {
    pub fn wgpu_type(&self) -> wgpu::BindingType {
        match self {
            ShaderBindingType::DynamicUniformBuffer => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: true,
                min_binding_size: None,
            },
            ShaderBindingType::UniformBuffer => wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            ShaderBindingType::Texture2D => wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
            },
            ShaderBindingType::Sampler => wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        }
    }
}

#[derive(Clone)]
pub struct ShaderBinding {
    pub(crate) binding: u32,
    pub(crate) binding_type: ShaderBindingType,
}

impl ShaderBinding {
    pub fn new(binding: u32, binding_type: ShaderBindingType) -> Self {
        Self { binding, binding_type }
    }

    pub fn wgpu_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: self.binding_type.wgpu_type(),
            count: None,
        }
    }
}

pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) vs_entry: String,
    pub(crate) fs_entry: String,
    pub(crate) bindings: HashMap<String, ShaderBinding>,
    pub(crate) inputs: HashMap<String, u32>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
}

impl Shader {
    pub fn new(renderer: &Renderer, source: &str) -> Self {
        Self::with_device(&*renderer.device, source)
    }

    pub(crate) fn with_device(device: &wgpu::Device, source: &str) -> Self {
        let module = naga::front::wgsl::parse_str(source).unwrap();

        let vs_entry = module.entry_points.iter().find(|&e| e.stage == naga::ShaderStage::Vertex).unwrap();
        let fs_entry = module.entry_points.iter().find(|&e| e.stage == naga::ShaderStage::Fragment).unwrap();

        let bindings = module
            .global_variables
            .iter()
            .filter_map(|(_, x)| match x.space {
                naga::AddressSpace::Uniform => {
                    let name = x.name.as_ref().unwrap().clone();
                    if name == "transform" {
                        Some((
                            name,
                            ShaderBinding::new(x.binding.as_ref().unwrap().binding, ShaderBindingType::DynamicUniformBuffer),
                        ))
                    } else {
                        Some((
                            name,
                            ShaderBinding::new(x.binding.as_ref().unwrap().binding, ShaderBindingType::UniformBuffer),
                        ))
                    }
                }
                naga::AddressSpace::Handle => {
                    let ty = module.types.get_handle(x.ty).unwrap();

                    let binding_type = match ty.inner {
                        naga::TypeInner::Sampler { .. } => ShaderBindingType::Sampler,
                        naga::TypeInner::Image {
                            dim: naga::ImageDimension::D2,
                            ..
                        } => ShaderBindingType::Texture2D,
                        _ => panic!(),
                    };

                    Some((
                        x.name.as_ref().unwrap().clone(),
                        ShaderBinding::new(x.binding.as_ref().unwrap().binding, binding_type),
                    ))
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let inputs = vs_entry
            .function
            .arguments
            .iter()
            .map(|x| match x.binding.as_ref().unwrap() {
                naga::Binding::Location { location, .. } => (x.name.as_ref().unwrap().clone(), *location),
                _ => panic!(),
            })
            .collect::<HashMap<_, _>>();

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let bind_group_entries = bindings.iter().map(|(_, x)| x.wgpu_entry()).collect::<Vec<_>>();

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &bind_group_entries,
            label: None,
        });

        Self {
            module,
            vs_entry: vs_entry.name.clone(),
            fs_entry: fs_entry.name.clone(),
            bindings,
            inputs,
            bind_group_layout,
        }
    }
}
