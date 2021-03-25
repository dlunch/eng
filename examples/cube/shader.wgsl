[[location(0)]]
var<in> in_position: vec4<f32>;
[[location(1)]]
var<in> in_tex_coord_vs: vec2<f32>;
[[location(0)]]
var<out> out_tex_coord: vec2<f32>;
[[builtin(position)]]
var<out> out_position: vec4<f32>;

[[block]]
struct mvp {
    transform: mat4x4<f32>;
};
[[group(0), binding(0)]]
var mvp: mvp;

[[stage(vertex)]]
fn vs_main() {
    out_position = mvp.transform * in_position;
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
    const mag: f32 = length(in_tex_coord_fs - vec2<f32>(0.5));
    const diffuse: vec4<f32> = textureSample(texture, sampler, in_tex_coord_fs);

    out_color = vec4<f32>(mix(diffuse.xyz, vec3<f32>(0.0), mag * mag), 1.0);
}
