use alloc::vec::Vec;

use hashbrown::HashMap;

use super::Renderer;

#[derive(Clone)]
pub enum ShaderBindingType {
    UniformBuffer,
    Texture2D,
    Sampler,
}

impl ShaderBindingType {
    pub fn wgpu_type(&self) -> wgpu::BindingType {
        match self {
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
            ShaderBindingType::Sampler => wgpu::BindingType::Sampler {
                comparison: false,
                filtering: true,
            },
        }
    }
}

#[derive(Clone)]
pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    pub fn wgpu_type(&self) -> wgpu::ShaderStages {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
        }
    }
}

#[derive(Clone)]
pub struct ShaderBinding {
    pub(crate) stage: ShaderStage,
    pub(crate) binding: u32,
    pub(crate) binding_type: ShaderBindingType,
}

impl ShaderBinding {
    pub fn new(stage: ShaderStage, binding: u32, binding_type: ShaderBindingType) -> Self {
        Self {
            stage,
            binding,
            binding_type,
        }
    }

    pub fn wgpu_entry(&self) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: self.binding,
            visibility: self.stage.wgpu_type(),
            ty: self.binding_type.wgpu_type(),
            count: None,
        }
    }
}

pub struct Shader {
    pub(crate) module: wgpu::ShaderModule,
    pub(crate) vs_entry: &'static str,
    pub(crate) fs_entry: &'static str,
    pub(crate) bindings: HashMap<&'static str, ShaderBinding>,
    pub(crate) inputs: HashMap<&'static str, u32>,
    pub(crate) bind_group_layout: wgpu::BindGroupLayout,
    pub(crate) pipeline_layout: wgpu::PipelineLayout,
}

impl Shader {
    pub fn new(
        renderer: &Renderer,
        source: &str,
        vs_entry: &'static str,
        fs_entry: &'static str,
        bindings: &[(&'static str, ShaderBinding)],
        inputs: &[(&'static str, u32)],
    ) -> Self {
        Self::with_device(&*renderer.device, source, vs_entry, fs_entry, bindings, inputs)
    }

    pub(crate) fn with_device(
        device: &wgpu::Device,
        source: &str,
        vs_entry: &'static str,
        fs_entry: &'static str,
        bindings: &[(&'static str, ShaderBinding)],
        inputs: &[(&'static str, u32)],
    ) -> Self {
        let module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });

        let bind_group_entries = bindings.iter().map(|(_, x)| x.wgpu_entry()).collect::<Vec<_>>();

        // TODO split bind groups by stage..
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &bind_group_entries,
            label: None,
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            push_constant_ranges: &[],
            bind_group_layouts: &[&bind_group_layout],
        });

        Self {
            module,
            vs_entry,
            fs_entry,
            bindings: bindings.iter().cloned().collect(),
            inputs: inputs.iter().cloned().collect(),
            bind_group_layout,
            pipeline_layout,
        }
    }
}
