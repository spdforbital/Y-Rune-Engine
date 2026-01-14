#version 460

layout (location = 0) in vec2 inUV;
layout (location = 0) out vec4 outColor;

layout (set = 0, binding = 0) readonly buffer SdfBuffer {
    float data[];
} sdf;

layout (push_constant) uniform PushConstants {
    mat4 inv_view;
    mat4 inv_proj;
    vec4 camera_pos;
    vec4 params; // x: time
    vec4 sun_dir;
    vec4 sun_color;
} pc;

const float MAX_DIST = 1000.0;
const int MAX_STEPS = 256;
const float SURF_DIST = 0.001;

// Operations
const float OP_SPHERE = 1.0;
const float OP_BOX = 2.0;
const float OP_CYLINDER = 3.0;
const float OP_TORUS = 4.0;
const float OP_UNION = 50.0;
const float OP_SUB = 51.0;
const float OP_INTERSECT = 52.0;
const float OP_SMOOTH_UNION = 53.0;
const float OP_SMOOTH_SUB = 54.0;
const float OP_SMOOTH_INTERSECT = 55.0;

// Struct to hold distance and material (color)
struct SdfResult {
    float dist;
    vec3 color;
};

// Distance Functions
float sdSphere(vec3 p, float s) {
    return length(p) - s;
}

float sdBox(vec3 p, vec3 b) {
    vec3 q = abs(p) - b;
    return length(max(q, 0.0)) + min(max(q.x, max(q.y, q.z)), 0.0);
}

float sdCylinder(vec3 p, float h, float r) {
    vec2 d = abs(vec2(length(p.xz), p.y)) - vec2(r, h);
    return min(max(d.x, d.y), 0.0) + length(max(d, 0.0));
}

float sdTorus(vec3 p, vec2 t) {
    vec2 q = vec2(length(p.xz) - t.x, p.y);
    return length(q) - t.y;
}

// Boolean Operators
SdfResult opUnion(SdfResult d1, SdfResult d2) {
    if (d1.dist < d2.dist) return d1;
    return d2;
}

SdfResult opSub(SdfResult d1, SdfResult d2) {
    // subtract d2 from d1: max(d1, -d2)
    // d1 is the main object, d2 is the cutter
    // Mix colors? usually just keep d1's color where it survives.
    // If we are inside d2, we essentially expose d2's interior surface?
    // Standard subtraction logic: distance is max(d1, -d2).
    // Color: if -d2.dist > d1.dist, we are on the surface of the "cut".
    // So if -d2.dist > d1.dist, return d2's color?
    // Actually, usually the cutter color is visible on the cut surface.
    float d = max(d1.dist, -d2.dist);
    vec3 c = d1.color;
    if (-d2.dist > d1.dist) c = d2.color;
    return SdfResult(d, c);
}

SdfResult opIntersect(SdfResult d1, SdfResult d2) {
    float d = max(d1.dist, d2.dist);
    vec3 c = d1.color;
    if (d2.dist > d1.dist) c = d2.color; // The surface is defined by the one with larger dist
    return SdfResult(d, c);
}

SdfResult opSmoothUnion(SdfResult d1, SdfResult d2, float k) {
    float h = clamp(0.5 + 0.5 * (d2.dist - d1.dist) / k, 0.0, 1.0);
    float d = mix(d2.dist, d1.dist, h) - k * h * (1.0 - h);
    vec3 c = mix(d2.color, d1.color, h);
    return SdfResult(d, c);
}

SdfResult opSmoothSub(SdfResult d1, SdfResult d2, float k) {
    float h = clamp(0.5 - 0.5 * (d2.dist + d1.dist) / k, 0.0, 1.0);
    float d = mix(d1.dist, -d2.dist, h) + k * h * (1.0 - h);
    vec3 c = mix(d1.color, d2.color, h);
    return SdfResult(d, c);
}


// Evaluator
SdfResult map(vec3 p) {
    // Stack for RPN
    const int STACK_SIZE = 16;
    SdfResult stack[STACK_SIZE];
    int sp = 0; // Stack pointer points to next free slot
    
    int ptr = 0;
    int len = sdf.data.length();
    
    // Safety break
    if (len == 0) return SdfResult(MAX_DIST, vec3(0.0));

    while (ptr < len) {
        float op = sdf.data[ptr];
        ptr++;

        if (op == OP_SPHERE) {
            vec3 c = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            float r = sdf.data[ptr++];
            vec3 col = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            
            float d = sdSphere(p - c, r);
            stack[sp++] = SdfResult(d, col);
        }
        else if (op == OP_BOX) {
            vec3 c = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            vec3 b = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            vec3 col = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            
            float d = sdBox(p - c, b);
            stack[sp++] = SdfResult(d, col);
        }
        else if (op == OP_CYLINDER) {
            vec3 c = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            float h = sdf.data[ptr++];
            float r = sdf.data[ptr++];
            vec3 col = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            
            float d = sdCylinder(p - c, h, r);
            stack[sp++] = SdfResult(d, col);
        }
        else if (op == OP_TORUS) {
             vec3 c = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            float r = sdf.data[ptr++];
            float t = sdf.data[ptr++];
            vec3 col = vec3(sdf.data[ptr++], sdf.data[ptr++], sdf.data[ptr++]);
            
            float d = sdTorus(p - c, vec2(r, t));
            stack[sp++] = SdfResult(d, col);
        }
        else if (op == OP_UNION) {
            if (sp >= 2) {
                SdfResult b = stack[--sp];
                SdfResult a = stack[--sp];
                stack[sp++] = opUnion(a, b);
            }
        }
        else if (op == OP_SUB) {
            if (sp >= 2) {
                // Stack is [..., A, B]. Sub is A - B.
                SdfResult b = stack[--sp];
                SdfResult a = stack[--sp];
                stack[sp++] = opSub(a, b);
            }
        }
        else if (op == OP_INTERSECT) {
            if (sp >= 2) {
                SdfResult b = stack[--sp];
                SdfResult a = stack[--sp];
                stack[sp++] = opIntersect(a, b);
            }
        }
        else if (op == OP_SMOOTH_UNION) {
             float k = sdf.data[ptr++];
             if (sp >= 2) {
                SdfResult b = stack[--sp];
                SdfResult a = stack[--sp];
                stack[sp++] = opSmoothUnion(a, b, k);
             }
        }
        else if (op == OP_SMOOTH_SUB) {
             float k = sdf.data[ptr++];
             if (sp >= 2) {
                SdfResult b = stack[--sp];
                SdfResult a = stack[--sp];
                stack[sp++] = opSmoothSub(a, b, k);
             }
        }
    }
    
    if (sp > 0) return stack[sp-1];
    return SdfResult(MAX_DIST, vec3(0.0));
}

vec3 calcNormal(vec3 p) {
    const float h = 0.0001;
    const vec2 k = vec2(1, -1);
    return normalize(
        k.xyy * map(p + k.xyy * h).dist +
        k.yyx * map(p + k.yyx * h).dist +
        k.yxy * map(p + k.yxy * h).dist +
        k.xxx * map(p + k.xxx * h).dist
    );
}

void main() {
    // 1. Ray setup
    vec2 uv = inUV * 2.0 - 1.0; // -1 to 1
    // Reconstruct world ray dir
    // NDC: (x, y, 0, 1) -> Unproject
    vec4 target = pc.inv_proj * vec4(uv.x, uv.y, 1.0, 1.0);
    vec4 dir_eye = vec4(normalize(target.xyz / target.w), 0.0);
    vec3 rd = normalize((pc.inv_view * dir_eye).xyz);
    vec3 ro = pc.camera_pos.xyz;

    float t = 0.0;
    SdfResult res;
    
    // 2. Raymarch
    int i = 0;
    for (i = 0; i < MAX_STEPS; i++) {
        vec3 p = ro + rd * t;
        res = map(p);
        
        // Check depth buffer? 
        // If t > linearized existing depth?
        // No, we are writing to gl_FragDepth. Vulkan will depth test for us IF we output correct depth.
        // But we are in a fullscreen quad. We need to respect existing geometry.
        // We can't access existing depth buffer easily unless we bind it as input attachment or texture.
        // So we just output depth and let Z-Test kill it?
        // BUT we are drawing a fullscreen quad at Z=0 (actually we didn't specify Z in vert).
        // Vert is at Z=0 or Z=1 depending on setup.
        // We need to validly kill the fragment if it's behind existing geometry.
        // If we enable Depth Test and set depth func to LESS, and we calculate gl_FragDepth.
        // It should work.
        
        if (res.dist < SURF_DIST) break;
        t += res.dist;
        if (t > MAX_DIST) break;
    }

    if (t > MAX_DIST) {
        discard;
    }

    // 3. Shading
    vec3 p = ro + rd * t;
    vec3 normal = calcNormal(p);
    vec3 sun_dir = normalize(pc.sun_dir.xyz);
    vec3 sun_color = pc.sun_color.rgb;
    
    float diff = max(dot(normal, sun_dir), 0.0);
    vec3 ambient = vec3(0.03);
    
    vec3 color = res.color * (ambient + diff * sun_color);
    
    // Fog?
    // color = applyFog(color, t);

    outColor = vec4(color, 1.0);
    
    // 4. Correct Depth
    // Project p back to NDC to get Z
    vec4 pp = inverse(pc.inv_proj * pc.inv_view) * vec4(p, 1.0); // Wait, proj * view * p
    // We only have inv_view/inv_proj. We need view/proj or just invert again?
    // Actually we can compute P_clip = Proj * View * P_world.
    // Inverse of (InvProj * InvView) is View * Proj.
    // This is expensive per fragment.
    // Easier: pass MVP or ViewProj in Push Constants?
    // But wait, existing pipeline logic uses Reverse Z? 
    // Or standard Z?
    // Let's assume standard 0..1.
    // Let's assume we can compute depth from T if projected.
    
    // Simple way:
    // P_view = View * P_world
    // P_clip = Proj * P_view
    // We can reconstruct it:
    // P_world is known.
    // Let's pass View/Proj as well, or simpler:
    // We have inv_view.
    // We can invert inv_view/inv_proj in shader but it's slow.
    // Let's just output manual depth?
    // Actually, Gl_FragDepth = z / w.
    // We really should pass the VP matrix.
    // I will ignore exact depth for now (it might fight with meshes) or just do best effort.
    // Or... I can invert the matrices in the CPU and pass them?
    // I'll update PushConstants to include `view_proj` matrix.
    
    // For now, let's try to get it working without perfect depth integration, 
    // or just assume it draws ON TOP of everything if I don't set depth?
    // If I don't set depth, it uses the Fullscreen Quad depth (which is 0 or 1).
    // If I set it to 0 (near), it covers everything.
    // I want it to occlude and be occluded.
    // I MUST write depth.
    
    // OK, let's just do a quick inverse here.
    mat4 view = inverse(pc.inv_view);
    mat4 proj = inverse(pc.inv_proj);
    vec4 p_clip = proj * view * vec4(p, 1.0);
    gl_FragDepth = p_clip.z / p_clip.w;
}
