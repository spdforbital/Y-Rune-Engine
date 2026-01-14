#version 460

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstants {
    mat4 inv_view_proj;
    vec4 camera_pos;
    float time;
    float intensity;
    vec2 pad;
} pc;

float hash(vec3 p) {
    return fract(sin(dot(p, vec3(12.9898, 78.233, 37.719))) * 43758.5453);
}

void main() {
    vec4 clip = vec4(v_uv * 2.0 - 1.0, 1.0, 1.0);
    vec4 world = pc.inv_view_proj * clip;
    vec3 world_pos = world.xyz / world.w;
    vec3 dir = normalize(world_pos - pc.camera_pos.xyz);

    float theta = atan(dir.z, dir.x);
    float phi = asin(clamp(dir.y, -1.0, 1.0));
    vec2 sky_uv = vec2(theta / (2.0 * 3.14159265) + 0.5, phi / 3.14159265 + 0.5);

    float grid = 220.0;
    vec2 gv = sky_uv * grid;
    vec2 cell = floor(gv);
    vec2 local = fract(gv) - 0.5;

    float h = hash(vec3(cell, 0.0));
    float star = step(0.996, h);
    float size = mix(0.05, 0.18, fract(h * 17.0));
    float dist = length(local);
    float core = smoothstep(size, 0.0, dist);
    float bright = star * core;

    float h2 = hash(vec3(cell, 11.7));
    float big = step(0.9993, h2);
    float big_core = smoothstep(0.35, 0.0, dist);
    bright += big * big_core * 2.0;

    float twinkle = 0.85 + 0.15 * sin(pc.time * 2.1 + h * 200.0);
    bright *= twinkle;

    float horizon = smoothstep(0.0, 0.2, dir.y);
    bright *= horizon;

    if (bright <= 0.001) {
        discard;
    }

    float color_t = fract(h * 7.13);
    vec3 cool = vec3(0.75, 0.85, 1.0);
    vec3 warm = vec3(1.0, 0.9, 0.75);
    vec3 color = mix(cool, warm, color_t);
    color *= (0.6 + 0.4 * fract(h * 13.7));

    float alpha = bright * pc.intensity;
    outColor = vec4(color, alpha);
}
