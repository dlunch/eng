pub trait Resource {
    fn wgpu_resource(&self) -> wgpu::BindingResource;
}
