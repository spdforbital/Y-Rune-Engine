#version 460
#extension GL_EXT_ray_query : require
#extension GL_EXT_nonuniform_qualifier : require

layout(location = 0) in vec2 v_uv;
layout(location = 1) in float v_material;
layout(location = 2) in vec3 v_normal;

// Helper for Normal Mapping
vec3 perturbNormal(vec3 N, vec3 p, vec2 uv, vec3 normal_sample) {
    vec3 map = normal_sample * 2.0 - 1.0;
    
    vec3 Q1 = dFdx(p);
    vec3 Q2 = dFdy(p);
    vec2 st1 = dFdx(uv);
    vec2 st2 = dFdy(uv);

    vec3 N_pert = normalize(N);
    vec3 T = normalize(Q1*st2.t - Q2*st1.t);
    // Gram-Schmidt
    T = normalize(T - N_pert * dot(N_pert, T));
    
    vec3 B = normalize(cross(N_pert, T));
    mat3 TBN = mat3(T, B, N_pert);

    return normalize(TBN * map);
}

layout(location = 3) in vec3 v_world;
layout(location = 4) in float v_opacity;
layout(location = 5) flat in uint v_mat_idx;
layout(location = 0) out vec4 outColor;

layout(binding = 4) uniform sampler2D global_textures[];
layout(binding = 7) uniform accelerationStructureEXT scene;

struct GpuMaterial {
    vec4 color;
    vec4 params; // metalness, roughness, padding, padding
    vec4 offset;
    vec4 base_position;
    vec4 rotation;
};

layout(binding = 10) readonly buffer Materials {
    GpuMaterial materials[];
};

// Forward+ Clustered Lighting
struct GpuLight {
    vec4 position;     // xyz = pos, w = radius
    vec4 direction;    // xyz = dir (spot), w = spot outer cos
    vec4 color;        // rgb = color, a = intensity
    uint light_type;
    float spot_inner;
    uint _pad0;
    uint _pad1;
};

struct ClusterInfo {
    uint offset;
    uint count;
};

layout(std430, binding = 11) readonly buffer LightBuffer {
    GpuLight lights[];
};

layout(std430, binding = 12) readonly buffer ClusterInfoBuffer {
    ClusterInfo cluster_info[];
};

layout(std430, binding = 13) readonly buffer LightIndexBuffer {
    uint light_indices[];
};

layout(std430, binding = 14) readonly buffer MeshletParamsBuffer {
    uint meshlet_model_indices[];
};

// Cluster parameters (could be push constants, but using constants for now)
const uint TILE_SIZE = 16;
const uint DEPTH_SLICES = 24;
const float NEAR_PLANE = 0.1;
const float FAR_PLANE = 1000.0;
const uint LIGHT_TYPE_POINT = 0;
const uint LIGHT_TYPE_SPOT = 1;

layout(push_constant) uniform PushConstants {
    mat4 mvp;
    uint meshlet_count;
    uint hovered_id; 
    float outline_width; 
    uint meshlet_offset;
    vec4 sun_dir;   // xyz direction, w intensity
    vec4 camera_pos;
    vec4 sun_color;
    uvec2 screen_size;
} pc;

const float PI = 3.14159265;

float distributionGGX(float NdotH, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float denom = (NdotH * NdotH) * (a2 - 1.0) + 1.0;
    return a2 / max(PI * denom * denom, 1e-4);
}

float geometrySchlickGGX(float NdotV, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    return NdotV / max(NdotV * (1.0 - k) + k, 1e-4);
}

float geometrySmith(float NdotV, float NdotL, float roughness) {
    return geometrySchlickGGX(NdotV, roughness) * geometrySchlickGGX(NdotL, roughness);
}

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float compute_shadow(vec3 origin, vec3 dir) {
    // Softer, shorter shadow trace to reduce popping.
    rayQueryEXT rq;
    rayQueryInitializeEXT(
        rq,
        scene,
        gl_RayFlagsTerminateOnFirstHitEXT | gl_RayFlagsOpaqueEXT,
        0xFF,
        origin,
        0.05,
        dir,
        200.0
    );
    while (rayQueryProceedEXT(rq)) {}
    bool hit = rayQueryGetIntersectionTypeEXT(rq, true) != gl_RayQueryCommittedIntersectionNoneEXT;
    // Blend hard shadow with a small amount of ambient light leakage.
    return hit ? 0.2 : 1.0;
}


// Hash function for randomness
float hash12(vec2 p) {
    vec3 p3  = fract(vec3(p.xyx) * .1031);
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.x + p3.y) * p3.z);
}

// Simple Ray Traced Ambient Occlusion
float compute_ao(vec3 origin, vec3 normal) {
    float occlusion = 0.0;
    const int SAMPLES = 4;
    const float RADIUS = 0.8;
    
    // Seed based on fragment position for noise
    float seed = hash12(gl_FragCoord.xy);
    
    for (int i = 0; i < SAMPLES; i++) {
        // Generate random vector in unit sphere
        float t = seed + float(i) * 0.3; // Pseudo-step
        vec3 rand_dir = vec3(
            fract(sin(t * 12.9898) * 43758.5453),
            fract(sin(t * 78.233) * 43758.5453),
            fract(sin(t * 54.53) * 43758.5453)
        ) * 2.0 - 1.0;
        
        // Orient to hemisphere
        if (dot(rand_dir, normal) < 0.0) {
            rand_dir = -rand_dir;
        }
        rand_dir = normalize(rand_dir);
        
        // Trace ray
        rayQueryEXT rq;
        rayQueryInitializeEXT(
            rq,
            scene,
            gl_RayFlagsTerminateOnFirstHitEXT | gl_RayFlagsOpaqueEXT,
            0xFF,
            origin,
            0.05, // Min t (bias)
            rand_dir,
            RADIUS
        );
        
        while (rayQueryProceedEXT(rq)) {}
        
        if (rayQueryGetIntersectionTypeEXT(rq, true) != gl_RayQueryCommittedIntersectionNoneEXT) {
            occlusion += 1.0;
        }
    }
    
    return 1.0 - (occlusion / float(SAMPLES));
}

// ============ CLUSTERED LIGHTING FUNCTIONS ============

// Get cluster index from fragment position and depth
uint get_cluster_index(vec2 frag_coord, float depth) {
    uint tile_x = uint(frag_coord.x) / TILE_SIZE;
    uint tile_y = uint(frag_coord.y) / TILE_SIZE;
    
    // Logarithmic depth slicing
    float log_depth = log(depth / NEAR_PLANE) / log(FAR_PLANE / NEAR_PLANE);
    uint slice = uint(clamp(log_depth * float(DEPTH_SLICES), 0.0, float(DEPTH_SLICES - 1)));
    
    // Dynamic cluster count based on screen size
    uint clusters_x = (pc.screen_size.x + TILE_SIZE - 1) / TILE_SIZE;
    uint clusters_y = (pc.screen_size.y + TILE_SIZE - 1) / TILE_SIZE;
    
    return tile_x + tile_y * clusters_x + slice * clusters_x * clusters_y;
}

// Evaluate a single point/spot light with PBR
vec3 evaluate_light(GpuLight light, vec3 world_pos, vec3 N, vec3 V, vec3 base_color, float metallic, float roughness) {
    vec3 light_pos = light.position.xyz;
    float radius = light.position.w;
    vec3 light_color = light.color.rgb;
    float intensity = light.color.a;
    
    // Vector from surface to light
    vec3 L = light_pos - world_pos;
    float dist = length(L);
    L = normalize(L);
    
    // Attenuation (inverse square with radius falloff)
    float attenuation = 1.0 / (dist * dist + 1.0);
    float falloff = clamp(1.0 - pow(dist / radius, 4.0), 0.0, 1.0);
    falloff = falloff * falloff;
    attenuation *= falloff;
    
    // Spot light cone attenuation
    if (light.light_type == LIGHT_TYPE_SPOT) {
        vec3 spot_dir = normalize(light.direction.xyz);
        float cos_angle = dot(-L, spot_dir);
        float cos_outer = light.direction.w;
        float cos_inner = light.spot_inner;
        float spot_effect = clamp((cos_angle - cos_outer) / (cos_inner - cos_outer), 0.0, 1.0);
        attenuation *= spot_effect;
    }
    
    if (attenuation < 0.001) return vec3(0.0);
    
    // PBR calculation
    float NdotL = max(dot(N, L), 0.0);
    float NdotV = max(dot(N, V), 0.0);
    vec3 H = normalize(V + L);
    float NdotH = max(dot(N, H), 0.0);
    float VdotH = max(dot(V, H), 0.0);
    
    float D = distributionGGX(NdotH, roughness);
    float G = geometrySmith(NdotV, NdotL, roughness);
    vec3 F0 = mix(vec3(0.04), base_color, metallic);
    vec3 F = fresnelSchlick(VdotH, F0);
    
    vec3 spec = (D * G * F) / max(4.0 * NdotV * NdotL, 0.0001);
    vec3 kd = (1.0 - F) * (1.0 - metallic);
    vec3 diffuse = kd * base_color / PI;
    
    vec3 radiance = light_color * intensity * attenuation;
    return (diffuse + spec) * radiance * NdotL;
}

// Evaluate all lights in this fragment's cluster
vec3 evaluate_cluster_lights(vec3 world_pos, vec3 N, vec3 V, vec3 base_color, float metallic, float roughness, float view_depth) {
    vec3 total_light = vec3(0.0);
    
    uint cluster_idx = get_cluster_index(gl_FragCoord.xy, view_depth);
    ClusterInfo info = cluster_info[cluster_idx];
    
    for (uint i = 0; i < info.count; i++) {
        uint light_idx = light_indices[info.offset + i];
        GpuLight light = lights[light_idx];
        total_light += evaluate_light(light, world_pos, N, V, base_color, metallic, roughness);
    }

    return total_light;
}

void main() {
    vec3 baseColor;
    float metallic;
    float roughness;
    float alpha = 1.0;
    vec3 normalMap = vec3(0.5, 0.5, 1.0);

    // Load from Material Buffer (Defaults)
    GpuMaterial mat = materials[v_mat_idx];
    baseColor = mat.color.rgb; // Convert to linear? Config is likely sRGB. Assume linear for now or engine handles it? 
    // Usually config colors are linear.
    metallic = mat.params.x;
    roughness = mat.params.y;

    if (v_material > 6.5) {
        baseColor = mat.color.rgb;
        metallic = mat.params.x;
        roughness = mat.params.y;
    } else if (v_material > 5.5) {
        // AK47 -> Index 4 (Albedo), 5 (MetallicSmoothness), 6 (Normal)
        baseColor = texture(global_textures[nonuniformEXT(4)], v_uv).rgb;
        vec4 metSmooth = texture(global_textures[nonuniformEXT(5)], v_uv);
        metallic = metSmooth.r;
        roughness = 1.0 - metSmooth.a;
        normalMap = texture(global_textures[nonuniformEXT(6)], v_uv).rgb;
    } else if (v_material > 4.5) {
        // Rock -> Index 3
        baseColor = texture(global_textures[nonuniformEXT(3)], v_uv).rgb;
        metallic = 0.0;
        roughness = 0.8;
    } else if (v_material > 2.5) {
        // Water
        if (v_material < 3.5) { // Water (3.0)
            baseColor = vec3(0.0, 0.3, 0.8);
            metallic = 0.1;
            roughness = 0.1;
            alpha = v_opacity;
        }
    } else if (v_material > 1.5) {
        // Tree -> Index 2
        baseColor = texture(global_textures[nonuniformEXT(2)], v_uv).rgb;
        metallic = 0.0;
        roughness = 0.8;
    } else if (v_material > 0.5) {
        // Concrete -> Index 0
        baseColor = texture(global_textures[nonuniformEXT(0)], v_uv).rgb;
        metallic = 0.0;
        roughness = 0.6;
    } else {
        // Grass -> Index 1
        baseColor = texture(global_textures[nonuniformEXT(1)], v_uv).rgb;
        metallic = 0.0;
        roughness = 0.5;
    }

    // For ground materials (grass/concrete), force a stable upward normal to avoid diagonal splits.
    vec3 N = normalize(v_normal);
    if (v_material < 1.5) {
        N = vec3(0.0, 1.0, 0.0);
    } else if (v_material > 5.5 && v_material < 6.5) {
         N = perturbNormal(N, v_world, v_uv, normalMap);
    }
    vec3 V = normalize(pc.camera_pos.xyz - v_world);
    vec3 L = normalize(-pc.sun_dir.xyz);
    float NdotL = max(dot(N, L), 0.0);
    float NdotV = max(dot(N, V), 0.0);

    float shadow = (pc.sun_dir.w > 0.001 && NdotL > 0.0) ? compute_shadow(v_world + N * 0.05, -pc.sun_dir.xyz) : 0.0;

    vec3 H = normalize(V + L);
    float NdotH = max(abs(dot(N, H)), 0.0);
    float VdotH = max(dot(V, H), 0.0);

    float D = distributionGGX(NdotH, roughness);
    float G = geometrySmith(NdotV, NdotL, roughness);
    vec3 F0 = mix(vec3(0.04), baseColor, metallic);
    vec3 F = fresnelSchlick(VdotH, F0);

    vec3 spec = (D * G * F) / max(4.0 * NdotV * NdotL, 1e-4);
    vec3 kd = (1.0 - F) * (1.0 - metallic);
    vec3 diffuse = kd * baseColor / PI;

    float light_strength = min(pc.sun_dir.w, 3.0); // clamp intensity to avoid blowouts
    vec3 radiance = pc.sun_color.rgb * light_strength;
    vec3 sun_lighting = (diffuse + spec) * radiance * NdotL * shadow;

    // Evaluate clustered point/spot lights
    float view_depth = length(pc.camera_pos.xyz - v_world);
    vec3 cluster_lighting = evaluate_cluster_lights(v_world, N, V, baseColor, metallic, roughness, view_depth);

    // Combine sun and clustered lights
    vec3 lighting = sun_lighting + cluster_lighting;

    vec3 ambient = baseColor * 0.06;
    
    // Apply RTAO
    float ao = compute_ao(v_world + N * 0.01, N);
    ambient *= ao;

    vec3 color = ambient + lighting;

    // Hover Outline / Rim Light (Internal)
    if (pc.outline_width == 0.0 && v_mat_idx == pc.hovered_id) {
        float rim = pow(1.0 - NdotV, 3.0);
        float rim_intensity = 0.5;
        vec3 rim_color = vec3(1.0); // White
        color += rim_color * rim * rim_intensity;
    }

    if (pc.outline_width > 0.0) {
        outColor = vec4(1.0, 1.0, 1.0, 1.0);
        return;
    }

    outColor = vec4(color, alpha);
}
