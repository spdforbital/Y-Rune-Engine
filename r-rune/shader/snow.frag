#version 460

layout(location = 0) in vec2 v_uv;
layout(location = 1) in float v_alpha;
layout(location = 2) in float v_rotation;

layout(location = 0) out vec4 outColor;

void main() {
    // Rotate UVs
    vec2 centered = v_uv - 0.5;
    float c = cos(v_rotation);
    float s = sin(v_rotation);
    mat2 rot_mat = mat2(c, -s, s, c);
    vec2 rotated = rot_mat * centered;
    
    // Soft circle shape
    float dist = length(rotated);
    // Smooth edges for soft look
    float shape = smoothstep(0.5, 0.2, dist);
    
    // Hot center
    float core = smoothstep(0.2, 0.0, dist);
    
    float alpha = v_alpha * shape;

    if (alpha <= 0.01) {
        discard;
    }

    // White snow color
    // Slightly bluish tint for realism in shadow/night?
    // Usually pure white 0.95-1.0 is good.
    vec3 color = vec3(0.95, 0.98, 1.0);
    
    outColor = vec4(color, alpha);
}
