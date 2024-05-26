struct Globals {
	row_x: vec4f,
	row_y: vec4f,
};

struct VertexInput {
	@location(0) position: vec2f,
	@location(1) color: vec4f,
	@location(2) uv: vec2f,
	@location(3) clip_rect: vec4u,
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
	
	return VertexOutput(
		vertex.color,
		vertex.clip_rect,
		vertex.uv,
		vec4f(clip_position, 0.5, 1.0)
	);
}

struct FragmentOutput {
	@location(0) color: vec4f,
	@location(0) @second_blend_source alpha: vec4f,
}


@fragment
fn fs_main(input: VertexOutput) -> FragmentOutput {
	let clip_rect = vec4f(input.clip_rect);
	let screen_pos = input.position.xy;

	if screen_pos.x < clip_rect.x
	|| screen_pos.x >= clip_rect.y
	|| screen_pos.y < clip_rect.z
	|| screen_pos.y >= clip_rect.w
	{
		discard;
	}

	// TODO(pat.m): Flag sampling operations as being either dual-source or regular rgba

	let tex_color = textureSample(text_atlas_tex, text_atlas_sampler, input.uv);
	let tex_alpha1 = textureSample(text_atlas_tex, text_atlas_sampler, input.uv - vec2f(0.5 / 2048.0, 0.0)).a;
	let tex_alpha2 = textureSample(text_atlas_tex, text_atlas_sampler, input.uv + vec2f(0.5 / 2048.0, 0.0)).a;

	// let alpha_alpha = input.color.a * tex_color.a;
	let alpha_alpha = input.color.a * (tex_alpha1 + tex_color.a + tex_alpha2) / 3.0;

	let color = vec3f(input.color.rgb * tex_color.rgb);
	let alpha = vec3f(input.color.a) * vec3f(tex_alpha1, tex_color.a, tex_alpha2);

	return FragmentOutput(
		vec4(color * alpha, alpha_alpha),
		vec4(alpha, 1.0),
	);
}