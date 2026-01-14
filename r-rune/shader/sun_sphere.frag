#version 460

layout(location = 0) in vec2 v_local;
layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstants {
    mat4 proj;
    vec4 sun_view_pos_radius;
    vec4 sun_color_intensity;
    vec4 params; // time, ray_strength, glow_radius, ray_length
} pc;

float hash(vec2 p) {
    p = fract(p * vec2(123.34, 456.21));
    p += dot(p, p + 45.32);
    return fract(p.x * p.y);
}

// 2D Noise
float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f);
    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// Fractional Brownian Motion
float fbm(vec2 p) {
    float total = 0.0;
    float amp = 0.5;
    for (int i = 0; i < 5; ++i) {
        total += noise(p) * amp;
        p *= 2.0;
        amp *= 0.5;
    }
    return total;
}

void main() {
    // Current pixel properties
    float r = length(v_local);
    float dist = r * 20.0; // Virtual distance scale
    
    // Sun core (The actual white-hot disk)
    float sun_radius = 0.8; // Adjusted relative to new quad size
    float sun_disk = 1.0 - smoothstep(sun_radius - 0.05, sun_radius + 0.05, dist);
    
    // Atmospheric Glow (Mie Scattering approximation)
    float glow = exp(-dist * 1.5) * 2.0;
    
    // God Rays / Corona
    float angle = atan(v_local.y, v_local.x);
    float time = pc.params.x;
    
    // Generate rays using polar coordinates and noise
    float rays = 0.0;
    
    // Layer 1: Slow, broad rays
    rays += noise(vec2(angle * 6.0 + time * 0.1, dist * 0.5 - time * 0.2)) * 0.5;
    
    // Layer 2: Fast, detailed rays
    rays += noise(vec2(angle * 20.0 - time * 0.3, dist * 2.0 + time * 0.5)) * 0.3;
    
    // Layer 3: FBM for organic detail
    rays += fbm(vec2(angle * 10.0, dist - time)) * 0.2;
    
    // Mask rays to fade out with distance and originate from center
    float ray_mask = smoothstep(0.0, 2.0, dist) * exp(-dist * 0.4); 
    rays *= ray_mask;

    // Combine Elements
    vec3 sun_color = vec3(1.0, 0.9, 0.7); // Warm center
    vec3 ray_color = vec3(1.0, 0.8, 0.4); // Golden rays
    
    vec3 col = vec3(0.0);
    
    // Add Core
    col += sun_disk * vec3(2.0); // Bright core
    
    // Add Glow
    col += glow * sun_color * 1.5;
    
    // Add Rays
    col += rays * ray_color * 2.5;

    // Intensity Multiplier
    float intensity = pc.sun_color_intensity.w * 2.0;
    col *= intensity;

    // Radial Fade for Quad Edges (prevent hard cuts)
    float alpha = smoothstep(1.0, 0.8, r);
    
    outColor = vec4(col, alpha);
}
