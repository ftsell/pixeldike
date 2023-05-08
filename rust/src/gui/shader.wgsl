@group(0) @binding(0)
var pixmap_texture: texture_2d<f32>;

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) i: u32,
) -> @builtin(position) vec4<f32> {
    let x = f32(1 - i32(i)) * 0.5;
    let y = f32(i32(i & 1u) * 2 - 1) * 0.5;
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(vertex_index) i: u32) -> FragmentOutput {
    var dimensions = textureDimensions(pixmap_texture);

    var coords = index2coords(i, dimensions);
    var texel = textureLoad(pixmap_texture);

    var out: FragmentOutput;
    out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    return out;
}

fn index2coords(i: u32, dimensions: vec2<u32>) -> vec2<u32> {
    var y = i % dimensions[0];
    var x = (i - row * dimensions[0]);
    return vec2(x, y);
}
