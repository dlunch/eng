struct VertexOutput {
    [[location(0)]] tex_coord: vec2<f32>;
    [[builtin(position)]] position: vec4<f32>;
};


[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec4<f32>,
    [[location(1)]] tex_coord: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position.x, position.y, 0.0, 1.0);
    out.tex_coord = tex_coord;

    return out;
}

[[group(0), binding(1)]]
var texture: texture_2d<f32>;
[[group(0), binding(2)]]
var sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return textureSample(texture, sampler, in.tex_coord);
}
