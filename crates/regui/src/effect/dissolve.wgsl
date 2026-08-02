// Break a child ui up instead of fading it out.
//
// A pattern gives every pixel a number from 0 to 1. Pixels whose number is below the
// threshold stay, the rest go, and a narrow band between the two fades.

struct Params {
    // The colour of the burning edge. Straight, not premultiplied; the alpha is how hard
    // the tint bites, and zero turns the burn off.
    burn: vec4<f32>,

    // Which way the wipe travels. Unit length, and unused by the noise pattern.
    direction: vec2<f32>,

    // The size of the target, in pixels.
    size: vec2<f32>,

    // Pattern values below this stay. It runs past both ends of 0 to 1, so that a whole
    // panel and an empty one are still reachable however wide the band is.
    threshold: f32,

    // How wide the fading band is, in pattern values.
    softness: f32,

    // The size of one noise cell, in pixels.
    cell: f32,

    // 1 to wipe, 0 to speckle.
    wipe: f32,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// A full-screen triangle pair, generated from the vertex index so that no vertex buffer is
// needed.
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VertexOutput {
    let uv = vec2<f32>(
        f32((index << 1u) & 2u),
        f32(index & 2u),
    );
    var out: VertexOutput;
    out.uv = uv;
    out.position = vec4<f32>(uv * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return out;
}

// One number per grid cell, spread over 0 to 1.
//
// Integer mixing rather than the usual `fract(sin(..))`, which drifts from one backend to
// the next and would make the speckles hardware dependent.
fn hash(cell: vec2<i32>) -> f32 {
    var h = u32(cell.x) * 374761393u + u32(cell.y) * 668265263u;
    h = (h ^ (h >> 13u)) * 1274126177u;
    h = h ^ (h >> 16u);
    return f32(h) / 4294967295.0;
}

// Smooth value noise: one number per cell, blended across the cell.
fn value_noise(p: vec2<f32>) -> f32 {
    let corner = vec2<i32>(floor(p));
    let f = fract(p);
    let w = f * f * (3.0 - 2.0 * f);

    let a = hash(corner);
    let b = hash(corner + vec2<i32>(1, 0));
    let c = hash(corner + vec2<i32>(0, 1));
    let d = hash(corner + vec2<i32>(1, 1));

    return mix(mix(a, b, w.x), mix(c, d, w.x), w.y);
}

// Speckles. The coarse octave breaks the panel into patches, the fine one tears their
// edges so that the front does not read as a smooth blob.
fn noise_pattern(uv: vec2<f32>) -> f32 {
    let p = uv * params.size / params.cell;
    return 0.65 * value_noise(p) + 0.35 * value_noise(p * 2.7 + vec2<f32>(17.3, 5.1));
}

// A gradient along the wipe direction, stretched to cover 0 to 1 exactly.
fn wipe_pattern(uv: vec2<f32>) -> f32 {
    let reach = params.size * params.direction;
    let low = min(reach.x, 0.0) + min(reach.y, 0.0);
    let high = max(reach.x, 0.0) + max(reach.y, 0.0);
    let here = dot(uv * params.size, params.direction);
    return clamp((here - low) / max(high - low, 1e-5), 0.0, 1.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let source_colour = textureSample(source, source_sampler, in.uv);

    var pattern = noise_pattern(in.uv);
    if params.wipe > 0.5 {
        pattern = wipe_pattern(in.uv);
    }

    // How far the pixel has crossed the band: 0 is still whole, 1 is already gone. The
    // clamp is what holds the two ends: at full progress the threshold sits at 1, so no
    // pattern value can be past the band, and at no progress it sits a whole band below 0,
    // so every value is.
    let edge = clamp((pattern - params.threshold) / params.softness, 0.0, 1.0);
    let keep = 1.0 - smoothstep(0.0, 1.0, edge);

    // A bump that peaks in the middle of the band and dies at both ends, so the burn shows
    // only on the front and never on a panel that is whole or gone.
    let burn = params.burn.a * 4.0 * edge * (1.0 - edge);

    // Everything is premultiplied, so the tint has to be scaled by the pixel's own alpha
    // to sit in the same space, and all four channels fade together.
    let tinted = mix(source_colour.rgb, params.burn.rgb * source_colour.a, burn);
    return vec4<f32>(tinted, source_colour.a) * keep;
}
