// A directional blur: one line of samples along a vector the caller gives.
//
// This is one pass, not two. A Gaussian separates into a horizontal pass and a vertical one
// because it is the same function on both axes; a smear along a free direction is not, so
// every sample has to be taken here.

struct Params {
    // How far apart to take samples, in texture coordinates.
    step: vec2<f32>,

    // Where the first sample sits, relative to the pixel, in texture coordinates.
    origin: vec2<f32>,

    // How many samples to take. At least one.
    samples: f32,
};

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;
@group(0) @binding(2) var<uniform> params: Params;

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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let count = i32(params.samples);

    // Every sample counts the same. A shutter is open for a fixed time, so each point on
    // the trail gets the same exposure; a falloff towards the tail looks softer but reads
    // as a glow around the child rather than as movement.
    //
    // Sum premultiplied alpha, which is what egui's textures hold. Averaging straight alpha
    // would drag colour out of transparent pixels and leave dark fringes along the trail.
    var sum = vec4<f32>(0.0);
    for (var i = 0; i < count; i = i + 1) {
        let offset = params.origin + params.step * f32(i);
        sum = sum + textureSample(source, source_sampler, in.uv + offset);
    }

    return sum / f32(count);
}
