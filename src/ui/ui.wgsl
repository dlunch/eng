struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

struct transform {
    model: mat4x4<f32>,
    view: mat4x4<f32>,
    projection: mat4x4<f32>,
};
@group(0) @binding(0)
var<uniform> transform: transform;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = transform.projection * transform.view * transform.model * position;
    out.tex_coord = tex_coord;

    return out;
}

@group(0) @binding(1)
var texture: texture_2d<f32>;
@group(0) @binding(2)
var textureSampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var diffuse: vec4<f32> = textureSample(texture, textureSampler, in.tex_coord);

    return vec4<f32>(diffuse.xyz, 1.0);
}