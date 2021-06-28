use hashbrown::HashMap;

use crate::Renderer;

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

pub enum ShaderStage {
    Vertex,
    Fragment,
}

impl ShaderStage {
    pub fn wgpu_type(&self) -> wgpu::ShaderStage {
        match self {
            ShaderStage::Vertex => wgpu::ShaderStage::VERTEX,
            ShaderStage::Fragment => wgpu::ShaderStage::FRAGMENT,
        }
    }
}

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
}

impl Shader {
    pub fn new(
        renderer: &Renderer,
        source: &str,
        vs_entry: &'static str,
        fs_entry: &'static str,
        bindings: HashMap<&'static str, ShaderBinding>,
        inputs: HashMap<&'static str, u32>,
    ) -> Self {
        let module = renderer.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(source.into()),
            flags: wgpu::ShaderFlags::default(),
        });

        Self {
            module,
            vs_entry,
            fs_entry,
            bindings,
            inputs,
        }
    }

    pub fn wgpu_bindings(&self) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> + '_ {
        self.bindings.iter().map(|(_, x)| x.wgpu_entry())
    }
}
