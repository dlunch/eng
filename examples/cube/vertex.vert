#version 450

layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 FragTexCoord;

layout(set = 0, binding = 0) uniform Locals {
    mat4 Transform;
};

void main() {
    FragTexCoord = TexCoord;
    gl_Position = Transform * Position;
}