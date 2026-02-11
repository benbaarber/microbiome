#version 100
precision highp float;

varying lowp vec2 uv;

uniform vec4 color;
uniform float glow_intensity;
uniform float time;
uniform float seed;
uniform vec2 gaze_offset;
uniform float heat; // 0 = cold, ~0.2 = resting cell, 1 = max heat

float hash(vec2 p) {
    return fract(sin(dot(p + seed, vec2(127.1, 311.7))) * 43758.5453);
}

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

float fbm(vec2 p) {
    float value = 0.0;
    float amplitude = 0.5;
    for (int i = 0; i < 4; i++) {
        value += amplitude * noise(p);
        p *= 2.0;
        amplitude *= 0.5;
    }
    return value;
}

void main() {
    vec2 centered = uv - vec2(0.5, 0.5);
    float dist = length(centered) * 2.0;

    if (dist > 1.0) {
        discard;
    }

    // Gaze offsets internals
    vec2 inner_centered = centered - gaze_offset * 0.15;
    float inner_dist = length(inner_centered) * 2.0;

    // looking up and down rotates internals
    float rotation = gaze_offset.y * 0.8;
    float cos_r = cos(rotation);
    float sin_r = sin(rotation);
    vec2 rotated_inner = vec2(
        inner_centered.x * cos_r - inner_centered.y * sin_r,
        inner_centered.x * sin_r + inner_centered.y * cos_r
    );
    float inner_angle = atan(rotated_inner.y, rotated_inner.x);

    float t = time;

    // === CORE ===
    float nucleus = 1.0 - smoothstep(0.0, 0.15, inner_dist);
    nucleus *= 0.5 * (0.7 + heat * 0.6);

    float core = 1.0 - smoothstep(0.0, 0.6, inner_dist);

    // === INTERNAL PATTERNS ===
    vec2 noiseCoord = centered * 7.0 + seed + vec2(t * 0.4, t * 0.3);
    float organicNoise = fbm(noiseCoord);

    vec2 noiseCoord2 = centered * 10.0 - seed * 2.0 + vec2(-t * 0.2, t * 0.25);
    float organicNoise2 = fbm(noiseCoord2) * 0.5;

    float totalNoise = (organicNoise + organicNoise2) * 0.45;

    // Concentric ripples
    float ripples = sin(inner_dist * 18.0 - t * 3.0 + seed) * 0.5 + 0.5;
    ripples *= smoothstep(0.7, 0.15, inner_dist);
    ripples *= 0.22;

    // Radial veins
    float veins = 0.0;
    float veinCount = 4.0 + mod(seed, 3.0);
    for (int i = 0; i < 9; i++) {
        if (float(i) >= veinCount) break;
        float fi = float(i);

        float baseAngle = fi * (6.28318 / veinCount) + seed * 0.5 + t * 0.08;

        float waveFreq = 2.0 + mod(seed + fi, 2.0);
        float waveAmp = 0.3 + 0.15 * sin(seed * fi);
        float wave = sin(inner_dist * waveFreq * 6.0 + t * 1.5 + fi * 2.0 + seed) * waveAmp;
        wave *= inner_dist;

        float wobble = noise(vec2(inner_dist * 4.0 + seed + fi, t * 0.5)) * 0.25;

        float veinAngle = baseAngle + wave + wobble;
        float angleDiff = abs(mod(inner_angle - veinAngle + 3.14159, 6.28318) - 3.14159);

        float baseWidth = 0.12 + 0.08 * (1.0 - inner_dist);
        float pulseWidth = baseWidth + 0.04 * sin(t * 2.5 + fi * 1.7 + seed);

        float flowSpeed = 3.0 + mod(seed, 2.0);
        float veinPulse = 0.6 + 0.4 * sin(inner_dist * 8.0 - t * flowSpeed + fi * 2.5);

        float vein = smoothstep(pulseWidth, 0.0, angleDiff);
        vein *= smoothstep(0.08, 0.25, inner_dist);
        vein *= smoothstep(0.85, 0.5, inner_dist);
        vein *= veinPulse;

        veins += vein;
    }
    veins *= 0.18;

    // === MEMBRANE ===
    float outerEdge = smoothstep(0.88, 0.91, dist) * (1.0 - smoothstep(0.93, 0.96, dist));
    float innerLayer = smoothstep(0.82, 0.86, dist) * (1.0 - smoothstep(0.88, 0.91, dist));
    float membraneGlow = smoothstep(0.95, 0.78, dist) * smoothstep(0.70, 0.80, dist);
    float membrane = outerEdge * 1.2 + innerLayer * 0.5 + membraneGlow * 0.25;

    // === OUTER GLOW ===
    float glow = (1.0 - smoothstep(0.75, 1.0, dist)) * glow_intensity * (0.5 + heat);

    // === COMBINE ===
    float brightness = core + nucleus + totalNoise * core + ripples + veins + membrane;
    float alpha = (brightness + glow) * color.a;

    vec3 centerColor = color.rgb * 1.5;
    vec3 edgeColor = color.rgb * 0.6;
    vec3 finalColor = mix(edgeColor, centerColor, core + nucleus);
    finalColor += (totalNoise - 0.3) * color.rgb * 0.5;

    // Heat tint: hot = warm orange, cold = cool blue
    vec3 warm_tint = vec3(1.0, 0.6, 0.2);
    vec3 cool_tint = vec3(0.3, 0.4, 0.8);
    float base_ratio = 0.2;
    if (heat > base_ratio) {
        float ht = (heat - base_ratio) / (1.0 - base_ratio);
        finalColor = mix(finalColor, finalColor * warm_tint, ht * 0.6);
    } else {
        float ct = 1.0 - heat / base_ratio;
        finalColor = mix(finalColor, finalColor * cool_tint, ct * 0.3);
    }

    gl_FragColor = vec4(finalColor, alpha);
}
