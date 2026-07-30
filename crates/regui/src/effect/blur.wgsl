// One direction of a separable Gaussian blur.
//
// Run it twice, once horizontally and once vertically, to blur in both directions for the
// cost of 2n taps instead of n².

struct Params {
    // How far apart to take samples, in texture coordinates. One axis is zero.
    step: vec2<f32>,

    // Standard deviation, in samples.
    sigma: f32,

    // How many samples to take on each side of the centre.
    radius: f32,
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Blur in premultiplied alpha, which is what egui's textures hold. Blurring
    // straight alpha would drag colour out of transparent pixels and leave dark fringes.
    var sum = textureSample(source, source_sampler, in.uv);
    var weight_sum = 1.0;

    let radius = i32(params.radius);
    let two_sigma_squared = 2.0 * params.sigma * params.sigma;

    for (var i = 1; i <= radius; i = i + 1) {
        let offset = f32(i);
        let weight = exp(-(offset * offset) / two_sigma_squared);
        sum = sum
            + weight * textureSample(source, source_sampler, in.uv + params.step * offset)
            + weight * textureSample(source, source_sampler, in.uv - params.step * offset);
        weight_sum = weight_sum + 2.0 * weight;
    }

    return sum / weight_sum;
}
