// Blurs whatever egui had already drawn, so a panel can sit on a blurred background.
//
// Three passes share this module: a horizontal blur, a vertical blur, and a final pass
// that draws the result back over the region egui asked for. Blurring in two 1D passes
// costs 2n samples per pixel instead of n².

struct Params {
    // How far apart to take samples, in texture coordinates. One axis is zero.
    step: vec2<f32>,

    // Standard deviation, in pixels.
    sigma: f32,

    // How many samples to take on each side of the centre.
    radius: f32,

    // The size of the whole target, in physical pixels, for turning a fragment position
    // back into a texture coordinate.
    target_size: vec2<f32>,

    // How far the edge of the glass fades out, in physical pixels. Zero for a hard edge.
    feather: f32,

    _padding: f32,

    // Laid over the blur, premultiplied. Use its alpha to fade the blur towards a colour.
    tint: vec4<f32>,

    // The rect being blurred, in physical pixels, for rounding off the corners.
    rect_min: vec2<f32>,
    rect_max: vec2<f32>,

    // Corner radii in physical pixels: north west, north east, south west, south east.
    corner_radii: vec4<f32>,
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
//
// For the blur passes this covers the whole target. For the final pass egui has narrowed
// the viewport down to the region being blurred, so the same triangle covers exactly that.
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
    // egui's textures hold premultiplied alpha, and that is what we blur. Blurring
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

// Signed distance to a rounded rectangle: negative inside, positive outside. `point` is
// relative to the rectangle's centre.
fn rounded_rect_distance(point: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let top = select(radii.x, radii.y, point.x > 0.0);
    let bottom = select(radii.z, radii.w, point.x > 0.0);
    let radius = select(top, bottom, point.y > 0.0);

    let corner = abs(point) - half_size + radius;
    return length(max(corner, vec2<f32>(0.0))) + min(max(corner.x, corner.y), 0.0) - radius;
}

// The viewport, not the vertex data, decides which part of the screen this covers, so work
// the texture coordinate back out from the fragment's position on the target.
@fragment
fn fs_draw(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let uv = position.xy / params.target_size;
    let blurred = textureSample(source, source_sampler, uv);

    // Both are premultiplied, so laying the tint over the blur is a plain `over`.
    let tinted = params.tint + blurred * (1.0 - params.tint.a);

    let centre = (params.rect_min + params.rect_max) * 0.5;
    let half_size = (params.rect_max - params.rect_min) * 0.5;
    let distance = rounded_rect_distance(position.xy - centre, half_size, params.corner_radii);

    // Fade out across the edge: one pixel by default, so the corners are not jagged, or the
    // whole feather if one was asked for. The fade straddles the edge, half of it outside
    // the rect and half inside. `smoothstep` rolls off at both ends, so a wide feather
    // reads as glass thinning out rather than as a linear ramp.
    let width = max(params.feather, 1.0);
    let ramp = clamp(0.5 - distance / width, 0.0, 1.0);

    // A linear ramp is what anti-aliasing wants, since it is measuring how much of the pixel
    // the shape covers. A feather is not measuring anything, so roll it off at both ends
    // instead: that reads as glass thinning out, where a linear ramp shows its two ends as
    // faint lines.
    let coverage = select(ramp, smoothstep(0.0, 1.0, ramp), params.feather > 1.0);

    // Premultiplied colour, so the coverage scales the alpha along with everything else.
    return tinted * coverage;
}
