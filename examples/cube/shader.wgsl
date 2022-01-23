struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};

struct transform {
    mvp: mat4x4<f32>;
};
[[group(0), binding(0)]]
var<uniform> mvp: transform;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = mvp.mvp * position;
    out.tex_coord = tex_coord;

    return out;
}

[[group(0), binding(1)]]
var texture: texture_2d<f32>;
[[group(0), binding(2)]]
var textureSampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var mag: f32 = length(in.tex_coord - vec2<f32>(0.5));
    var diffuse: vec4<f32> = textureSample(texture, textureSampler, in.tex_coord);
    var a: vec3<f32> = vec3<f32>(mag * mag, mag * mag, mag * mag);

    return vec4<f32>(mix(diffuse.xyz, vec3<f32>(0.0), a), 1.0);
}
