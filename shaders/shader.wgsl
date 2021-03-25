[[location(0)]]
var<in> in_position: vec4<f32>;
[[location(1)]]
var<in> in_tex_coord_vs: vec2<f32>;
[[location(0)]]
var<out> out_tex_coord: vec2<f32>;
[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[stage(vertex)]]
fn vs_main() {
    out_position = vec4<f32>(in_position.x, in_position.y, 0.0, 1.0);
    out_tex_coord = in_tex_coord_vs;
}

[[location(0)]]
var<in> in_tex_coord_fs: vec2<f32>;
[[location(0)]]
var<out> out_color: vec4<f32>;
[[group(0), binding(1)]]
var texture: texture_2d<f32>;
[[group(0), binding(2)]]
var sampler: sampler;

[[stage(fragment)]]
fn fs_main() {
    out_color = textureSample(texture, sampler, in_tex_coord_fs);
}
