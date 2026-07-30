// One direction of a separable Gaussian blur over a child ui's rendered image.
//
// The backdrop blur has its own shader because it also has to tint, round off corners and
// work out where on the target it is. Here the source and the target are the same size and
// we blur the whole thing, so all of that goes away.

struct Params {
    // How far apart to take samples, in texture coordinates. One axis is zero.
    step: vec2<f32>,

    // Standard deviation, in pixels.
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

// One oversized triangle, built from the vertex index so no vertex buffer is needed. Its
// texture coordinates run 0 to 2, so the part that survives clipping covers exactly 0 to 1.
// Draw it with three vertices.
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
fn fs_blur(in: VertexOutput) -> @location(0) vec4<f32> {
    // egui renders premultiplied alpha, and that is what we blur. Blurring straight alpha
    // would drag colour out of transparent pixels and leave dark fringes, which matters
    // here because a child ui is mostly transparent.
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
