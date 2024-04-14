struct Globals {
    row_x: vec4<f32>,
    row_y: vec4<f32>,
};

struct VertexOutput {
    @location(0) v_color: vec4<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;

@vertex
fn vs_main(@location(0) a_position: vec2<f32>, @location(1) a_color: vec4<f32>) -> VertexOutput {
    let transform = transpose(mat2x3(globals.row_x.xyw, globals.row_y.xyw));

    let position = transform * vec3f(a_position, 1.0);

    return VertexOutput(
        a_color,
        vec4(position, 0.5, 1.0)
    );
}

@fragment
fn fs_main(@location(0) v_color: vec4<f32>) -> @location(0) vec4<f32> {
    return v_color;
}