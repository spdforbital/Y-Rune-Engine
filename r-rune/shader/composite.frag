#version 460

layout (binding = 0) uniform sampler2D sceneColor;
layout (binding = 1) uniform sampler2D bloomBlur;

layout (location = 0) in vec2 inUV;
layout (location = 0) out vec4 outColor;

layout (push_constant) uniform PushConstants {
    float bloom_strength;
    uint bloom_enabled;
} pc;

vec3 aces_tonemap(vec3 color) {
    mat3 m1 = mat3(
        0.59719, 0.07600, 0.02840,
        0.35458, 0.90834, 0.13383,
        0.04823, 0.01566, 0.83777
    );
    mat3 m2 = mat3(
        1.60475, -0.10208, -0.00327,
        -0.53108,  1.10813, -0.07276,
        -0.07367, -0.00605,  1.07602
    );
    vec3 v = m1 * color;
    vec3 a = v * (v + 0.0245786) - 0.000090537;
    vec3 b = v * (0.983729 * v + 0.4329510) + 0.238081;
    return clamp(m2 * (a / b), 0.0, 1.0);
}

void main() {
    vec3 color = texture(sceneColor, inUV).rgb;
    
    if (pc.bloom_enabled > 0) {
        vec3 bloom = texture(bloomBlur, inUV).rgb;
        color += bloom * pc.bloom_strength;
    }
    
    // Tone mapping
    color = aces_tonemap(color);
    
    // Gamma correction
    // color = pow(color, vec3(1.0 / 2.2)); // Swapchain might handle sRGB? 
    // Usually manual gamma if writing to UNORM. If writing to SRGB view, hardware handles it.
    // The engine likely uses SRGB Swapchain Image View.
    // So we output Linear if we want hardware to convert, or we output sRGB if view is UNORM.
    // src/vulkan/vk_swapchain.rs uses `B8G8R8A8_SRGB`.
    // So we should output LINEAR color, and GPU converts to sRGB.
    // So NO manual gamma here.
    
    outColor = vec4(color, 1.0);
}
