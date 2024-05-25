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
    @location(2) uv: vec2f,
    @builtin(position) position: vec4f,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(0) @binding(1) var text_atlas_tex: texture_2d<f32>;
@group(0) @binding(2) var text_atlas_sampler: sampler;

@vertex
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let transform = transpose(mat2x3(globals.row_x.xyw, globals.row_y.xyw));

    let clip_position = transform * vec3f(vertex.position, 1.0);

    // Premultiply vertex color
    let color = vertex.color;
    let color_premul = vec4f(color.rgb * color.a, color.a);

    let uv = vec2f(1.0, 1.0);

    return VertexOutput(
        color_premul,
        vertex.clip_rect,
        uv,
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

    let tex_color = textureSample(text_atlas_tex, text_atlas_sampler, input.uv);
    let tex_color_premul = vec4f(tex_color.rgb * tex_color.a, tex_color.a);

    return input.color * tex_color_premul;
}