#version 460

layout(location = 0) in vec3 v_localPos; // Interpolated local position [-0.5, 0.5]
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler3D densityTex;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    vec4 camera_pos; // World space
    float time;
    vec3 pad;
} pc;

// Simple blackbody ramp
vec3 blackbody(float temp) {
    vec3 color = vec3(255, 255, 255);
    color *= temp; // Simple gain
    // R, G, B ramps could be better using a texture or curve
    // Physics based approximation:
    if (temp < 0.1) return vec3(0.0);
    if (temp < 0.3) return vec3(1.0, 0.0, 0.0) * temp * 3.0;
    if (temp < 0.6) return mix(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), (temp - 0.3) / 0.3);
    return mix(vec3(1.0, 1.0, 0.0), vec3(1.0, 1.0, 1.0), (temp - 0.6) / 0.4);
}

// Better 
vec3 fireColor(float d) {
    return mix(vec3(1.0, 0.0, 0.0), vec3(1.0, 1.0, 0.0), d);
}

void main() {
    // We are inside a cube. v_localPos is the exit point or entry point?
    // Since we render the cube, the rasterizer gives us the surface position.
    
    // We need to raymarch THROUGH the volume.
    // 1. Calculate ray direction in local space.
    // Ideally we pass camera pos in local space to simplify.
    // But we have camera pos in world space.
    // We need Model Matrix (Inverse) to transform Camera -> Local.
    // But we passed MVP... 
    
    // Workaround:
    // We render the FRONT faces of the cube (if outside) or BACK faces (if inside)?
    // Standard approach: Render Back faces to get "Exit Point", compute Entry by ray-box intersection?
    // Or just intersect box in the shader.
    
    // Let's assume we are viewing from outside for now (simple case).
    // The fragment IS the entry point (front face).
    // Ray direction: normalize(WorldPos - CameraPos).
    // Transform RayDir to Local Space?
    // Need ModelInv.
    
    // ALTERNATIVE:
    // Just use v_localPos as the 3D texture coordinate?
    // This draws the SURFACE of the cube with the texture value at that point.
    // That's a sliced view, not volumetric.
    
    // VOLUMETRIC RAYMARCHING:
    // Start = v_localPos (Entry point on front face).
    // Direction = View Ray.
    // End = Ray Box Exit.
    
    // To get View Ray in LOCAL space, we need the camera position relative to the cube center, rotated/scaled correctly.
    // Since I hardcoded the Model Matrix in Rust to: 
    // Translate(-2, 1, 5) * Scale(2).
    // Local Space [-0.5, 0.5] * 2 + Center = World.
    // WorldNode = (Local * 2) + Center.
    // Local = (World - Center) / 2.
    
    vec3 center = vec3(-2.0, 1.0, 5.0);
    float scale = 2.0;
    
    vec3 localCam = (pc.camera_pos.xyz - center) / scale;
    
    vec3 rayDir = normalize(v_localPos - localCam);
    
    // March
    vec3 curr = v_localPos;
    float stepSize = 0.02; // 50 steps
    vec4 acc = vec4(0.0);
    
    for (int i = 0; i < 64; i++) {
        // Texture Coord: [-0.5, 0.5] -> [0, 1]
        vec3 uvw = curr + 0.5;
        
        if (uvw.x < 0 || uvw.x > 1 || uvw.y < 0 || uvw.y > 1 || uvw.z < 0 || uvw.z > 1) break;
        
        vec4 data = texture(densityTex, uvw);
        float density = data.r; // Fire
        float smoke = data.g;   // Smoke
        
        // Fire emission
        if (density > 0.01) {
            vec3 col = blackbody(density);
            // Additive blend for fire
            acc.rgb += col * density * 0.5 * (1.0 - acc.a);
            acc.a += density * 0.1;
        }
        
        // Smoke absorption
        if (smoke > 0.01) {
             float a = smoke * 0.2;
             vec3 smokeColor = vec3(0.1, 0.1, 0.1); // Dark grey
             acc.rgb = mix(acc.rgb, smokeColor, a * (1.0 - acc.a)); // Alpha blend
             acc.a += a;
        }

        if (acc.a >= 0.95) break;
        
        curr += rayDir * stepSize;
    }
    
    outColor = acc;
}
