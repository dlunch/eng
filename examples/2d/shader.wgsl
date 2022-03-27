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

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.5, 0.5, 0.5, 1.0);
}
