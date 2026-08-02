// A drop shadow thrown by whatever the child actually is.
//
// Three passes. Two blur the child's alpha, one direction each, the way `blur.wgsl` does.
// The third lays the sharp child over the blurred alpha, coloured and offset.
//
// All three share one set of bindings and one uniform struct, because a shader module can
// only bind a slot once. The blur passes bind the same texture to both slots and ignore the
// second.

struct Params {
    // The shadow's colour, premultiplied, in the target's own colour space.
    color: vec4<f32>,

    // How far apart to take blur samples, in texture coordinates. One axis is zero.
    step: vec2<f32>,

    // How far to throw the shadow, in texture coordinates.
    offset: vec2<f32>,

    // Standard deviation of the blur, in samples.
    sigma: f32,

    // How many blur samples to take on each side of the centre.
    radius: f32,

    // Spread: how much to grow the blurred shape. See `gain` in `shadow.rs`.
    gain: f32,
};

@group(0) @binding(0) var front: texture_2d<f32>;
@group(0) @binding(1) var shadow: texture_2d<f32>;
@group(0) @binding(2) var texture_sampler: sampler;
@group(0) @binding(3) var<uniform> params: Params;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// A full-screen triangle, generated from the vertex index so that no vertex buffer is
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

// One direction of the blur. Only alpha is carried, so the shadow takes the child's shape
// and not its colour.
@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    var sum = textureSample(front, texture_sampler, in.uv).a;
    var weight_sum = 1.0;

    let taps = i32(params.radius);
    let two_sigma_squared = 2.0 * params.sigma * params.sigma;

    for (var i = 1; i <= taps; i = i + 1) {
        let offset = f32(i);
        let weight = exp(-(offset * offset) / two_sigma_squared);
        sum = sum
            + weight * textureSample(front, texture_sampler, in.uv + params.step * offset).a
            + weight * textureSample(front, texture_sampler, in.uv - params.step * offset).a;
        weight_sum = weight_sum + 2.0 * weight;
    }

    // Scaling the blurred alpha moves its half-way line outwards, which grows the shape.
    let alpha = clamp(params.gain * sum / weight_sum, 0.0, 1.0);

    // The colour is left black: nothing reads this texture but its alpha.
    return vec4<f32>(0.0, 0.0, 0.0, alpha);
}

// The sharp child over its shadow.
@fragment
fn fs_composite(in: VertexOutput) -> @location(0) vec4<f32> {
    // Throwing the shadow one way means reading the blur the other way. The sampler clamps,
    // but the padding keeps the blur clear of the edge, so it clamps to nothing.
    let alpha = textureSample(shadow, texture_sampler, in.uv - params.offset).a;

    // Both sides are premultiplied, so laying the child over the shadow is a plain `over`.
    let shaded = params.color * alpha;
    let child = textureSample(front, texture_sampler, in.uv);
    return child + shaded * (1.0 - child.a);
}
