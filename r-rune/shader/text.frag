#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;

layout(binding = 0) uniform sampler2D fontTex;

void main() {
    vec4 texColor = texture(fontTex, v_uv);
    outColor = texColor;
    // outColor = vec4(1.0, 0.0, 0.0, 1.0); // Force red
}
