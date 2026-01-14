#version 450
#extension GL_EXT_ray_query : require

layout(location = 0) in vec3 v_normal;
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_world;
layout(location = 0) out vec4 outColor;

layout(binding = 5) uniform sampler2D texBase;
layout(binding = 6) uniform accelerationStructureEXT scene;

layout(push_constant) uniform SkinnedPC {
    mat4 mvp;
    vec4 sun_dir;
    vec4 sun_color;
    vec4 model_pos_scale;
    vec4 model_rot;
} pc;

float compute_shadow(vec3 origin, vec3 dir) {
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
    return hit ? 0.2 : 1.0;
}

void main() {
    vec3 albedo = texture(texBase, v_uv).rgb;
    vec3 N = normalize(v_normal);
    vec3 L = normalize(-pc.sun_dir.xyz);
    float ndotl = max(dot(N, L), 0.0);
    float light_strength = min(pc.sun_dir.w, 2.5);
    float shadow = (pc.sun_dir.w > 0.001 && ndotl > 0.0)
        ? compute_shadow(v_world + N * 0.05, -pc.sun_dir.xyz)
        : 0.0;
    vec3 light = pc.sun_color.rgb * light_strength * ndotl * shadow;
    vec3 ambient = albedo * 0.2;
    vec3 color = albedo * light + ambient;
    outColor = vec4(color, 1.0);
}
