struct VertexOutput {
    @location(0) color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0)
@binding(1)
var<storage> transform_matrices: array<mat4x4<f32>>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) color: vec4<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.color = color;
    result.position = transform * (transform_matrices[0] * position);
    return result;
}

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vertex.color;
}

@fragment
fn fs_wire(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 0.5);
}
