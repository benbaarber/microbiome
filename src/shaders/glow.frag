#version 100
precision highp float;

varying lowp vec2 uv;

uniform vec4 color;
uniform float glow_intensity;
uniform float time;
uniform float seed;
uniform vec2 gaze_offset; // Normalized direction the cell is "looking"

// Simple pseudo-random function
float hash(vec2 p) {
    return fract(sin(dot(p + seed, vec2(127.1, 311.7))) * 43758.5453);
}

// Smooth noise
float noise(vec2 p) {
    vec2 i = floor(p);
    vec2 f = fract(p);
    f = f * f * (3.0 - 2.0 * f); // smoothstep

    float a = hash(i);
    float b = hash(i + vec2(1.0, 0.0));
    float c = hash(i + vec2(0.0, 1.0));
    float d = hash(i + vec2(1.0, 1.0));

    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
}

// Fractal brownian motion for organic texture
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
    float angle = atan(centered.y, centered.x);

    if (dist > 1.0) {
        discard;
    }

    // Internal elements are offset by gaze (scaled to UV space)
    vec2 inner_centered = centered - gaze_offset * 0.15;
    float inner_dist = length(inner_centered) * 2.0;

    // Looking up/down rotates the internals smoothly
    float rotation = gaze_offset.y * 0.8;
    float cos_r = cos(rotation);
    float sin_r = sin(rotation);
    vec2 rotated_inner = vec2(
        inner_centered.x * cos_r - inner_centered.y * sin_r,
        inner_centered.x * sin_r + inner_centered.y * cos_r
    );

    float inner_angle = atan(rotated_inner.y, rotated_inner.x);

    // Animated time factor
    float t = time;

    // === CORE STRUCTURE (uses offset coordinates) ===
    // Bright nucleus in center
    float nucleus = 1.0 - smoothstep(0.0, 0.15, inner_dist);
    nucleus *= 0.5;

    // Main body with tighter falloff
    float core = 1.0 - smoothstep(0.0, 0.6, inner_dist);

    // === INTERNAL PATTERNS (uses original coordinates - stays fixed like boundary) ===
    // Organic noise texture inside the cell - animated and seeded
    vec2 noiseCoord = centered * 7.0 + seed + vec2(t * 0.4, t * 0.3);
    float organicNoise = fbm(noiseCoord);

    // Secondary noise layer for more complexity
    vec2 noiseCoord2 = centered * 10.0 - seed * 2.0 + vec2(-t * 0.2, t * 0.25);
    float organicNoise2 = fbm(noiseCoord2) * 0.5;

    float totalNoise = (organicNoise + organicNoise2) * 0.45;

    // Concentric ripples (pulsing internal rings)
    float ripples = sin(inner_dist * 18.0 - t * 3.0 + seed) * 0.5 + 0.5;
    ripples *= smoothstep(0.7, 0.15, inner_dist); // fade ripples toward edge
    ripples *= 0.22;

    // Radial veins/structures emanating from center - unique per cell
    float veins = 0.0;
    float veinCount = 6.0 + mod(seed, 3.0);
    for (int i = 0; i < 9; i++) {
        if (float(i) >= veinCount) break;
        float fi = float(i);

        // Base angle for this vein
        float baseAngle = fi * (6.28318 / veinCount) + seed * 0.5 + t * 0.08;

        // Wavy distortion - vein curves as it extends outward
        float waveFreq = 2.0 + mod(seed + fi, 2.0); // Different frequency per vein
        float waveAmp = 0.3 + 0.15 * sin(seed * fi); // Different amplitude per vein
        float wave = sin(inner_dist * waveFreq * 6.0 + t * 1.5 + fi * 2.0 + seed) * waveAmp;
        wave *= inner_dist; // Wave increases toward edge

        // Add some noise-based wobble
        float wobble = noise(vec2(inner_dist * 4.0 + seed + fi, t * 0.5)) * 0.25;

        float veinAngle = baseAngle + wave + wobble;
        float angleDiff = abs(mod(inner_angle - veinAngle + 3.14159, 6.28318) - 3.14159);

        // Width varies along length - thicker near center, thinner at edges
        float baseWidth = 0.12 + 0.08 * (1.0 - inner_dist);
        float pulseWidth = baseWidth + 0.04 * sin(t * 2.5 + fi * 1.7 + seed);

        // Flowing pulse along the vein
        float flowSpeed = 3.0 + mod(seed, 2.0);
        float veinPulse = 0.6 + 0.4 * sin(inner_dist * 8.0 - t * flowSpeed + fi * 2.5);

        // Vein visibility - fade in from center, fade out at edge
        float vein = smoothstep(pulseWidth, 0.0, angleDiff);
        vein *= smoothstep(0.08, 0.25, inner_dist); // fade in from center
        vein *= smoothstep(0.85, 0.5, inner_dist);  // fade out at edge
        vein *= veinPulse;

        veins += vein;
    }
    veins *= 0.18;

    // === MEMBRANE ===
    // Perfect circle with layered shading for depth

    // Outer bright edge (highlight)
    float outerEdge = smoothstep(0.88, 0.91, dist) * (1.0 - smoothstep(0.93, 0.96, dist));

    // Inner softer layer
    float innerLayer = smoothstep(0.82, 0.86, dist) * (1.0 - smoothstep(0.88, 0.91, dist));

    // Subtle gradient glow leading into membrane
    float membraneGlow = smoothstep(0.95, 0.78, dist) * smoothstep(0.70, 0.80, dist);

    // Combine layers with different intensities
    float membrane = outerEdge * 1.2 + innerLayer * 0.5 + membraneGlow * 0.25;


    // === OUTER GLOW ===
    float glow = (1.0 - smoothstep(0.75, 1.0, dist)) * glow_intensity;

    // === COMBINE ===
    float brightness = core + nucleus + totalNoise * core + ripples + veins + membrane;
    float alpha = (brightness + glow) * color.a;

    // Color variation: slightly shift hue toward center
    vec3 centerColor = color.rgb * 1.5; // brighter center
    vec3 edgeColor = color.rgb * 0.6;   // darker edge
    vec3 finalColor = mix(edgeColor, centerColor, core + nucleus);

    // Add color variation from animated noise
    finalColor += (totalNoise - 0.3) * color.rgb * 0.5;

    gl_FragColor = vec4(finalColor, alpha);
}
