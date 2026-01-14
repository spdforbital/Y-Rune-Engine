#version 450

layout(location = 0) in vec3 v_world; // World position from mesh shader
layout(location = 0) out vec4 outColor;

layout(push_constant) uniform PushConstants {
    vec4 camera_pos;
    vec4 sun_dir;
    vec4 sun_color;
    mat4 inv_view_proj;
    mat4 mvp; // Match push constant layout!
    vec4 wind_params;
    float time;
    float cloud_scale;
    float cloud_density;
    float cloud_absorption;
    float min_height;
    float max_height;
} pc;

// Constants
const int STEPS = 32;
const int LIGHT_STEPS = 8;
const float BAYER_FACTOR = 1.0/16.0;

// Bayer matrix for dithering
const float bayer[16] = float[](
    0.0 * BAYER_FACTOR, 8.0 * BAYER_FACTOR, 2.0 * BAYER_FACTOR, 10.0 * BAYER_FACTOR,
    12.0 * BAYER_FACTOR, 4.0 * BAYER_FACTOR, 14.0 * BAYER_FACTOR, 6.0 * BAYER_FACTOR,
    3.0 * BAYER_FACTOR, 11.0 * BAYER_FACTOR, 1.0 * BAYER_FACTOR, 9.0 * BAYER_FACTOR,
    15.0 * BAYER_FACTOR, 7.0 * BAYER_FACTOR, 13.0 * BAYER_FACTOR, 5.0 * BAYER_FACTOR
);

// Hash function for noise
float hash(vec3 p) {
    p = fract(p * 0.3183099 + .1);
    p *= 17.0;
    return fract(p.x * p.y * p.z * (p.x + p.y + p.z));
}

// 3D Value Noise
float noise(vec3 x) {
    vec3 i = floor(x);
    vec3 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    return mix(mix(mix(hash(i + vec3(0, 0, 0)), hash(i + vec3(1, 0, 0)), f.x),
                   mix(hash(i + vec3(0, 1, 0)), hash(i + vec3(1, 1, 0)), f.x), f.y),
               mix(mix(hash(i + vec3(0, 0, 1)), hash(i + vec3(1, 0, 1)), f.x),
                   mix(hash(i + vec3(0, 1, 1)), hash(i + vec3(1, 1, 1)), f.x), f.y), f.z);
}

// Fractional Brownian Motion
float fbm(vec3 p) {
    float f = 0.0;
    float amp = 0.5;
    for(int i = 0; i < 4; i++) {
        f += amp * noise(p);
        p *= 2.0;
        amp *= 0.5;
    }
    return f;
}

vec3 windOffset() {
    vec2 wind_dir = pc.wind_params.xy;
    float wind_len = length(wind_dir);
    vec2 wind_norm = wind_len > 1e-3 ? wind_dir / wind_len : vec2(1.0, 0.0);
    float wind_speed = pc.wind_params.z;
    return vec3(wind_norm.x, 0.0, wind_norm.y) * pc.time * wind_speed * 0.01;
}

// Ray-Sphere intersection
vec2 raySphereDst(vec3 sphereCenter, float sphereRadius, vec3 rayOrigin, vec3 rayDir) {
    vec3 oc = rayOrigin - sphereCenter;
    float b = dot(oc, rayDir);
    float c = dot(oc, oc) - sphereRadius * sphereRadius;
    float h = b * b - c;
    if (h < 0.0) return vec2(-1.0); // No intersection
    h = sqrt(h);
    return vec2(-b - h, -b + h);
}

// Ray-Plane intersection
float rayPlaneDst(vec3 rayOrigin, vec3 rayDir, float planeHeight) {
    float t = (planeHeight - rayOrigin.y) / rayDir.y;
    return t > 0.0 ? t : -1.0;
}

// Cloud density function
float getDensity(vec3 p) {
    // Simple height gradient
    float heightFraction = (p.y - pc.min_height) / (pc.max_height - pc.min_height);
    if(heightFraction < 0.0 || heightFraction > 1.0) return 0.0;
    
    // Wind and movement
    // Use pc.time directly as it is now pre-scaled wind time
    vec3 animatedP = p * pc.cloud_scale + windOffset();
    
    float noiseVal = fbm(animatedP);
    
    // Shape clouds
    float bottomFade = smoothstep(0.0, 0.2, heightFraction);
    float topFade = smoothstep(1.0, 0.8, heightFraction);
    
    float density = noiseVal * bottomFade * topFade * pc.cloud_density;
    return max(0.0, density - 0.3); // Threshold
}

// Phase function (Henyey-Greenstein)
float hg(float a, float g) {
    float g2 = g * g;
    return (1.0 - g2) / (4.0 * 3.14159 * pow(1.0 + g2 - 2.0 * g * a, 1.5));
}

const float EARTH_RADIUS = 600000.0; // Scaled down earth for game feel (600km)

void main() {
    float SPHERE_INNER = EARTH_RADIUS + pc.min_height;
    float SPHERE_OUTER = EARTH_RADIUS + pc.max_height;
    
    vec3 rayOrigin = pc.camera_pos.rgb;
    vec3 rayDir = normalize(v_world - rayOrigin);
    
    // Sphere center relative to world origin (0,0,0) with earth below
    // We assume the "Earth" follows the camera horizontally to simulate infinite earth
    vec3 sphereCenter = vec3(pc.camera_pos.x, -EARTH_RADIUS, pc.camera_pos.z);
    
    vec2 tInner = raySphereDst(sphereCenter, SPHERE_INNER, rayOrigin, rayDir);
    vec2 tOuter = raySphereDst(sphereCenter, SPHERE_OUTER, rayOrigin, rayDir);
    
    // Determine start and end of the cloud volume along the ray
    float distToStart = 0.0;
    float distToEnd = 0.0;
    
    // Case 1: We are below the clouds (ground)
    // We should hit inner sphere first (tInner.y), then outer sphere (tOuter.y)
    // raySphereDst returns (-1, -1) if no hit. t.x is first hit, t.y is second.
    
    // Valid hits must be > 0.
    
    // Simplify: The cloud layer is the volume between Inner and Outer spheres.
    // Interval [max(0, tInner.y), max(0, tOuter.y)] ? 
    // Usually looking up: enter inner at tInner.y, exit outer at tOuter.y.
    
    // Fix raySphere implementation to filter negative t
    // If inside sphere, one t is negative, one positive.
    
    // Robust Logic:
    // Determine interval overlapping [0, infinity) with [tInner.x, tInner.y] and [tOuter.x, tOuter.y].
    // Actually, looking up from below:
    // Ray intersects inner sphere at two points? No, if we are inside inner sphere, intersects at 1 point?
    // Wait, camera y=1.2. Inner Sphere Radius = R+150. Center y=-R.
    // Camera dist to center = R + 1.2.
    // We are INSIDE inner sphere.
    // So ray intersects inner sphere at tInner.y (exit). tInner.x is negative (behind us).
    // Ray intersects outer sphere at tOuter.y (exit).
    // So cloud volume is from tInner.y to tOuter.y.
    
    float tStart = max(0.0, tInner.y);
    float tEnd = max(0.0, tOuter.y);
    
    // If looking down, tInner.y might be very far (other side of earth).
    // Restrict max distance to avoid rendering other side of planet.
    if (tEnd > 100000.0) tEnd = 100000.0;
    if (tStart > 100000.0) tStart = 100000.0;

    distToStart = tStart;
    distToEnd = tEnd;
    
    if (distToEnd <= distToStart) {
        outColor = vec4(0.0);
        return;
    }
    
    float traceDist = distToEnd - distToStart;
    float stepSize = traceDist / float(STEPS);
    
    // Dithering
    int x = int(gl_FragCoord.x) % 4;
    int y = int(gl_FragCoord.y) % 4;
    float dither = bayer[y * 4 + x];
    
    vec3 currentPos = rayOrigin + rayDir * (distToStart + stepSize * dither);
    
    float transmittance = 1.0;
    vec3 lightEnergy = vec3(0.0);
    vec3 sunDir = normalize(pc.sun_dir.xyz);
    float cosTheta = dot(rayDir, sunDir);
    float phaseVal = hg(cosTheta, 0.5); 
    
    for (int i = 0; i < STEPS; i++) {
        if (transmittance < 0.01) break;
        
        // Calculate spherical altitude
        float distToCenter = distance(currentPos, sphereCenter);
        float altitude = distToCenter - EARTH_RADIUS;
        
        if (altitude < pc.min_height || altitude > pc.max_height) {
            currentPos += rayDir * stepSize;
            continue;
        }

        // Density function adapted for spherical altitude
        // Reuse getDensity logic but override height calculation inside?
        // Or better, inline the density logic here or pass altitude.
        
        // Let's call a modified version or just reimplement essential part
        float heightFraction = (altitude - pc.min_height) / (pc.max_height - pc.min_height);
        
        // Wind calc
        vec3 animatedP = currentPos * pc.cloud_scale + windOffset();
        float noiseVal = fbm(animatedP);
        float bottomFade = smoothstep(0.0, 0.2, heightFraction);
        float topFade = smoothstep(1.0, 0.8, heightFraction);
        float density = max(0.0, noiseVal * bottomFade * topFade * pc.cloud_density - 0.3);
        
        if (density > 0.0) {
             // Lighting
             // Simple light attenuation
             float lightTransmittance = 1.0;
             // Approximate light path density
             // (Ideally march to sun, but expensive. Use density + heuristic)
             float lightDepth = density * 10.0; // Fake
             lightTransmittance = exp(-lightDepth * pc.cloud_absorption);
             
             vec3 scattering = density * lightTransmittance * pc.sun_color.rgb * pc.sun_dir.w * phaseVal;
             float stepTransmittance = exp(-density * stepSize * pc.cloud_absorption);
             lightEnergy += scattering * transmittance * (1.0 - stepTransmittance);
             transmittance *= stepTransmittance;
        }
        
        currentPos += rayDir * stepSize;
    }
    
    // Natural fade at horizon due to thickness/atmosphere can be simulated by simple distance Fog
    // if needed. But spherical geometry should handle the geometric cutoff.
    // Let's add slight atmospheric blend?
    // For now, raw clouds.
    
    outColor = vec4(lightEnergy, 1.0 - transmittance);
}
