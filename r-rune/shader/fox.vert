#version 450

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_uv;

layout(location = 0) out vec3 v_normal;
layout(location = 1) out vec2 v_uv;

layout(push_constant) uniform SkinnedPC {
    mat4 mvp;
    vec4 sun_dir;
    vec4 sun_color;
} pc;

void main() {
    gl_Position = pc.mvp * vec4(in_pos, 1.0);
    v_normal = in_normal;
    v_uv = in_uv;
}
