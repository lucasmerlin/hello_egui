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

    // How far the rim bends what it shows, in physical pixels. Zero switches it off.
    refraction: f32,

    // How bright the lit rim is, from 0 to 1. Zero switches it off.
    specular: f32,

    // How thick the pane is, in physical pixels. Both rim effects live in a band this wide.
    thickness: f32,

    _padding_2: f32,

    // Unit vector pointing at the light, in screen space. y grows downward.
    light: vec2<f32>,

    _padding_3: vec2<f32>,

    // Superellipse power. Zero uses the rounded rectangle and its corner radii instead.
    squircle: f32,

    // How hard the rim squeezes what is behind the pane, from 0 to 1. Zero switches it off.
    lens: f32,

    // How bright the sheen running round the rim is. Zero switches it off.
    sheen: f32,

    // How much grain is laid over the glass, from 0 to 1. Zero switches it off.
    grain: f32,

    // How far the colours split where the lens bends. Zero switches it off.
    dispersion: f32,

    // Three separate floats, not a `vec3`: a `vec3` is aligned to 16 bytes, which would
    // round the whole struct up past the 36 floats the Rust side writes.
    _padding_4: f32,
    _padding_5: f32,
    _padding_6: f32,
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

// Signed distance to a superellipse filling the rect: negative inside, positive outside.
//
// `|x/hx|^n + |y/hy|^n = 1` is the shape; at `n == 2` it is an ellipse, and it squares up as
// `n` grows. Around 4 it is the rounded square Apple's panels use, where the straight sides
// run into the corners with no join to see.
//
// The implicit equation is zero on the boundary but says nothing about how far away a point
// is, so divide it by the length of its own gradient. That turns it into a distance in
// pixels, and doing the division in pixels rather than in the normalised square is what
// keeps the edge one pixel wide on a rect that is wider than it is tall.
fn superellipse_distance(point: vec2<f32>, half_size: vec2<f32>, power: f32) -> f32 {
    let size = max(half_size, vec2<f32>(0.001));
    let normalised = abs(point) / size;

    let value = pow(normalised.x, power) + pow(normalised.y, power) - 1.0;
    let gradient = power * pow(normalised, vec2<f32>(power - 1.0)) / size;

    return value / max(length(gradient), 1e-6);
}

// The pane's outline, whichever shape was asked for.
fn shape_distance(point: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> f32 {
    if params.squircle > 0.0 {
        return superellipse_distance(point, half_size, params.squircle);
    }
    return rounded_rect_distance(point, half_size, radii);
}

// How steep the glass is at this point of the rim.
//
// The pane is modelled as a flat sheet with a quarter-round bevel ground into its edge.
// `height` is 1 at the rim and 0 where the bevel meets the flat, so the slope of that
// quarter circle runs away at the rim; cap it, then scale the peak back to 1.
fn bevel_slope(height: f32) -> f32 {
    const CAP: f32 = 0.6;
    return height / max(sqrt(1.0 - height * height), CAP) * CAP;
}

// Which way the glass faces, pointing out of the shape.
//
// The gradient of a signed distance field is a unit vector away from the surface, so
// difference the field a pixel each way. In the middle of the pane the two rim effects are
// already zero, so the flat spot in the gradient there costs us nothing.
fn surface_normal(point: vec2<f32>, half_size: vec2<f32>, radii: vec4<f32>) -> vec2<f32> {
    let step = vec2<f32>(1.0, 0.0);
    let gradient = vec2<f32>(
        shape_distance(point + step.xy, half_size, radii)
            - shape_distance(point - step.xy, half_size, radii),
        shape_distance(point + step.yx, half_size, radii)
            - shape_distance(point - step.yx, half_size, radii),
    );
    let length_squared = dot(gradient, gradient);
    return select(vec2<f32>(0.0), gradient * inverseSqrt(length_squared), length_squared > 1e-8);
}

// How far in towards the centre the rim pulls what is behind the pane.
//
// A thick lens does not bend its edge a little: it gathers a whole band of the background
// into a thin ring, so the ring shows a squeezed copy of what lies around the pane. This is
// the curve that does it. It is 1 over almost all of the pane, so the middle looks straight
// through, and falls to about a quarter at the very rim. `depth` is 0 at the edge and 1 at
// the deepest point inside.
//
// The constants are the ones from OverShifted's LiquidGlass, which are tuned rather than
// derived; the shape of the curve is what matters, not the numbers.
fn lens_shrink(depth: f32) -> f32 {
    const E: f32 = 2.718281828459045;
    const A: f32 = 0.7;
    const B: f32 = 2.3;
    const C: f32 = 5.2;
    const D: f32 = 6.9;
    const POWER: f32 = 3.0;

    let curve = 1.0 - B * pow(C * E, -D * depth - A);
    return pow(max(curve, 0.0), POWER);
}

// Grain, from an integer hash of the pixel.
//
// `fract(sin(..))` is the usual one-liner, but it leans on how a backend rounds `sin` at
// large arguments, so the same frame can come out different on two machines. This does not.
fn grain(position: vec2<f32>) -> f32 {
    // The parentheses are not optional: WGSL leaves the precedence of `^` against `*`
    // undefined, so Tint (and thus the browser) rejects the expression without them, even
    // though naga accepts it natively.
    var hash = (u32(position.x) * 73856093u) ^ (u32(position.y) * 19349663u);
    hash = hash ^ (hash >> 13u);
    hash = hash * 1274126177u;
    hash = hash ^ (hash >> 16u);
    return f32(hash) / 4294967295.0 - 0.5;
}

// The viewport, not the vertex data, decides which part of the screen this covers, so work
// the texture coordinate back out from the fragment's position on the target.
@fragment
fn fs_draw(@builtin(position) position: vec4<f32>) -> @location(0) vec4<f32> {
    let centre = (params.rect_min + params.rect_max) * 0.5;
    let half_size = (params.rect_max - params.rect_min) * 0.5;
    let point = position.xy - centre;
    let distance = shape_distance(point, half_size, params.corner_radii);

    // A band of the pane's own thickness on each side of the edge. Straddling the edge is
    // what makes this compose with a feather: the feather moves the visible edge outwards,
    // and the rim follows it instead of hiding inside the fade.
    let thickness = max(params.thickness, 0.001);
    let height = clamp(1.0 - abs(distance) / thickness, 0.0, 1.0);
    let slope = bevel_slope(height);

    let normal = surface_normal(point, half_size, params.corner_radii);

    // How far inside the pane this pixel is, from 0 at the rim to 1 at the deepest point.
    let reach = max(min(half_size.x, half_size.y), 0.001);
    let depth = clamp(-distance / reach, 0.0, 1.0);

    // Pull the sample in towards the centre. `lens` mixes the curve in, so a strength of
    // zero leaves the point where it was and costs nothing.
    let shrink = mix(1.0, lens_shrink(depth), params.lens);

    // A thick edge shows you what is beside the pane, not what is under it, so walk the
    // sample outwards along the surface. The middle of the pane keeps looking straight
    // down, which is what tells the eye the glass has depth.
    let bend = normal * (params.refraction * slope);
    let bent = centre + point * shrink + bend;
    var blurred = textureSample(source, source_sampler, bent / params.target_size);

    // Glass does not bend every colour by the same amount, so a hard bend fringes. Take the
    // red and the blue from a little further along the same path. `1 - shrink` is zero
    // wherever the lens is doing nothing, so the fringe stays where the bending is.
    if params.dispersion > 0.0 {
        let split = params.dispersion * (1.0 - shrink);
        let red = centre + point * (shrink - split) + bend;
        let blue = centre + point * (shrink + split) + bend;
        blurred.r = textureSample(source, source_sampler, red / params.target_size).r;
        blurred.b = textureSample(source, source_sampler, blue / params.target_size).b;
    }

    // Both are premultiplied, so laying the tint over the blur is a plain `over`.
    let tinted = params.tint + blurred * (1.0 - params.tint.a);

    // Light the rim from one side. Cubing the facing term keeps the highlight to the arc
    // that really turns towards the light, and squaring the slope keeps it to a line along
    // the very edge. A highlight spread over the whole band reads as a glow, not as glass.
    let facing = max(dot(normal, params.light), 0.0);
    let highlight = params.specular * slope * slope * facing * facing * facing;

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

    // A sheen running round the rim, bright on one side and dark on the other. Where the
    // specular is a point of light, this is the whole edge picking up the room, which is
    // what stops a wide rim reading as a flat grey band. It only lives in the outermost
    // sliver of the pane.
    const RIM: f32 = 0.06;
    let angle = atan2(point.y, point.x) - 0.5;
    let sheen = params.sheen * sin(angle) * (1.0 - smoothstep(0.0, RIM, depth));

    // Grain, so a large sheet of glass is not perfectly smooth. Scaled by the alpha, so it
    // stays premultiplied and does not show up outside the shape.
    let speck = params.grain * grain(position.xy) * tinted.a;

    // White, and opaque where it is bright, so it stays premultiplied.
    let lit = tinted * (1.0 + sheen) + vec4<f32>(highlight) + vec4<f32>(speck, speck, speck, 0.0);

    // Premultiplied colour, so the coverage scales the alpha along with everything else.
    // The mask goes on last, so no part of the rim can spill outside the shape.
    return lit * coverage;
}
