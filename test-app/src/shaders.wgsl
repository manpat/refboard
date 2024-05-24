struct Globals {
    row_x: vec4f,
    row_y: vec4f,
};

struct VertexInput {
    @location(0) position: vec2f,
    @location(1) color: vec4f,
    @location(2) clip_rect: vec4u,
};

struct VertexOutput {
    @location(0) color: vec4f,
    @location(1) @interpolate(flat) clip_rect: vec4u,
    @builtin(position) position: vec4f,
};

@group(0) @binding(0) var<uniform> globals: Globals;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let transform = transpose(mat2x3(globals.row_x.xyw, globals.row_y.xyw));

    let clip_position = transform * vec3f(vertex.position, 1.0);

    return VertexOutput(
        vertex.color,
        vertex.clip_rect,
        vec4f(clip_position, 0.5, 1.0)
    );
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4f {
    let clip_rect = vec4f(input.clip_rect);
    let screen_pos = input.position.xy;

    if screen_pos.x < clip_rect.x
    || screen_pos.x >= clip_rect.y
    || screen_pos.y < clip_rect.z
    || screen_pos.y >= clip_rect.w
    {
        discard;
    }

    return input.color;
}