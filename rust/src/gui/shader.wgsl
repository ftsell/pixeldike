struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

type FragmentInput = VertexOutput;

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = vertex.color;
    out.clip_position = vec4<f32>(vertex.position, 0.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: FragmentInput) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = vec4<f32>(in.color, 1.0);
    return out;
}
