#version 460

layout(location = 0) out vec2 v_local;

layout(push_constant) uniform PushConstants {
    mat4 proj;
    vec4 sun_view_pos_radius;
    vec4 sun_color_intensity;
    vec4 params;
} pc;

void main() {
    vec2 corners[6] = vec2[](
        vec2(-1.0, -1.0),
        vec2( 1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0, -1.0),
        vec2( 1.0,  1.0),
        vec2(-1.0,  1.0)
    );

    vec2 local = corners[gl_VertexIndex];
    v_local = local;

    vec3 pos = pc.sun_view_pos_radius.xyz + vec3(local * pc.sun_view_pos_radius.w, 0.0);
    gl_Position = pc.proj * vec4(pos, 1.0);
}
