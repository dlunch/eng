#version 450

layout(location = 0) in vec2 FragTexCoord;
layout(location = 0) out vec4 OutColor;
layout(set = 0, binding = 1) uniform texture2D Texture;
layout(set = 0, binding = 2) uniform sampler Sampler;

void main() {
    float mag = length(FragTexCoord - vec2(0.5));
    vec4 diffuse = texture(sampler2D(Texture, Sampler), FragTexCoord);

    OutColor = vec4(mix(diffuse.xyz, vec3(0.0), mag * mag), 1.0);
}