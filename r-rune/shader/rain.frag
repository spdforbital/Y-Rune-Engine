#version 460

layout(location = 0) in vec2 v_uv;
layout(location = 1) in float v_alpha;

layout(location = 0) out vec4 outColor;

void main() {
    float side = abs(v_uv.x - 0.5) * 2.0;
    float body = smoothstep(1.0, 0.0, side);
    float head = smoothstep(0.0, 0.15, v_uv.y);
    float tail = smoothstep(1.0, 0.85, v_uv.y);
    float alpha = v_alpha * body * head * tail;

    if (alpha <= 0.001) {
        discard;
    }

    float core = smoothstep(0.6, 0.0, side);
    vec3 color = vec3(0.68, 0.78, 0.88) * (0.8 + 0.4 * core);
    outColor = vec4(color, alpha);
}
