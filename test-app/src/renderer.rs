use crate::prelude::*;

// use wgpu::util::DeviceExt;
use std::sync::Arc;

pub struct Renderer {
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,

	framebuffer: wgpu::TextureView,

	text_atlas_texture: wgpu::Texture,

	globals_buffer: wgpu::Buffer,
	vector_bind_group: wgpu::BindGroup,
	vector_render_pipeline: wgpu::RenderPipeline,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	vertex_bytes: u64,
	index_bytes: u64,

	msaa_samples: u32,
}

impl Renderer {
	#[instrument(skip_all)]
	pub async fn start(window: Arc<winit::window::Window>) -> anyhow::Result<Renderer> {
		let size = window.inner_size();

		// create an instance
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

		// create an surface
		let surface = instance.create_surface(window)?;

		println!("surface created");

		// create an adapter
		let Some(adapter) = instance.request_adapter(&wgpu::RequestAdapterOptions {
			// power_preference: wgpu::PowerPreference::HighPerformance,
			power_preference: wgpu::PowerPreference::LowPower,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false,
		}).await
			else {
				anyhow::bail!("Failed to request adapter")
			};

		println!("adapter created");

		// create a device and a queue
		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				label: None,
				required_features: wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES
					| wgpu::Features::DUAL_SOURCE_BLENDING,
				required_limits: wgpu::Limits::default(),
			},
			None,
		)
		.await?;

		println!("device created");

		let mut surface_config = surface.get_default_config(&adapter, size.width, size.height).ok_or_else(|| anyhow::format_err!("Failed to get surface config"))?;
		surface_config.present_mode = wgpu::PresentMode::AutoNoVsync;
		surface.configure(&device, &surface_config);

		println!("surface configured");


		// Set up render pipeline
		let shader = device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));
		
		let globals_buffer_byte_size = std::mem::size_of::<Globals>() as u64;
		let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float{ filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 2,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		let supported_sample_counts = adapter.get_texture_format_features(swapchain_format).flags.supported_sample_counts();
		let msaa_samples = supported_sample_counts.into_iter().max().unwrap_or(1);

		println!("Using MSAA x{msaa_samples}");

		// Premultiplied dual-source blending
		let blend_state = wgpu::BlendState {
			color: wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::OneMinusSrc1,
				operation: wgpu::BlendOperation::Add,
			},
			alpha: wgpu::BlendComponent {
				src_factor: wgpu::BlendFactor::One,
				dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
				operation: wgpu::BlendOperation::Add,
			},
		};

		let vector_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex>() as u64,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &wgpu::vertex_attr_array![
						0 => Float32x2,
						1 => Float32x4,
						2 => Float32x2,
						3 => Uint16x4,
					],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: swapchain_format,
					write_mask: wgpu::ColorWrites::all(),
					blend: Some(blend_state),
				})],
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				polygon_mode: wgpu::PolygonMode::Fill,
				front_face: wgpu::FrontFace::Ccw,
				strip_index_format: None,
				cull_mode: Some(wgpu::Face::Back),
				conservative: false,
				unclipped_depth: false,
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: msaa_samples,
				.. wgpu::MultisampleState::default()
			},
			multiview: None,
		});

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: 8<<20,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: 8<<20,
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Globals ubo"),
			size: globals_buffer_byte_size,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let text_atlas_texture = device.create_texture(&wgpu::TextureDescriptor {
			label: Some("Text Atlas"),
			size: wgpu::Extent3d {
				// TODO(pat.m): should come from limits/TextAtlas
				width: ui::TEXT_ATLAS_SIZE,
				height: ui::TEXT_ATLAS_SIZE,
				depth_or_array_layers: 1,
			},
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Rgba8UnormSrgb,
			usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb]
		});

		let text_atlas_texture_view = text_atlas_texture.create_view(&wgpu::TextureViewDescriptor::default());

		let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("Text Sampler"),
			min_filter: wgpu::FilterMode::Linear,
			mag_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,

			.. Default::default()
		});

		let vector_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Bind group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(globals_buffer.as_entire_buffer_binding()),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::TextureView(&text_atlas_texture_view),
				},
				wgpu::BindGroupEntry {
					binding: 2,
					resource: wgpu::BindingResource::Sampler(&atlas_sampler),
				},
			],
		});

		let framebuffer = Self::create_framebuffer(&device, &surface_config, msaa_samples);

		{
			let image_copy = wgpu::ImageCopyTexture {
				texture: &text_atlas_texture,
				origin: wgpu::Origin3d {
					x: text_atlas_texture.width() - 4,
					y: text_atlas_texture.height() - 1,
					z: 0,
				},

				mip_level: 0,
				aspect: wgpu::TextureAspect::All,
			};

			let data_layout = wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4*4),
				rows_per_image: None,
			};

			let size = wgpu::Extent3d {
				width: 4,
				height: 1,
				depth_or_array_layers: 1,
			};

			// Write single white pixel for non-textured geometry
			queue.write_texture(image_copy, &[255; 4 * 4], data_layout, size);
		}

		Ok(Renderer {
			device,
			queue,
			surface,
			surface_config,

			globals_buffer,
			vector_bind_group,
			vector_render_pipeline,

			framebuffer,

			text_atlas_texture,

			vertex_buffer,
			index_buffer,

			vertex_bytes: 0,
			index_bytes: 0,

			msaa_samples,
		})
	}

	fn create_framebuffer(device: &wgpu::Device, surface_conf: &wgpu::SurfaceConfiguration, sample_count: u32) -> wgpu::TextureView {
		let multisampled_texture_extent = wgpu::Extent3d {
			width: surface_conf.width,
			height: surface_conf.height,
			depth_or_array_layers: 1,
		};

		let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
			label: None,
			size: multisampled_texture_extent,
			mip_level_count: 1,
			sample_count,
			dimension: wgpu::TextureDimension::D2,
			format: surface_conf.format,
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			view_formats: &[],
		};

		device
			.create_texture(multisampled_frame_descriptor)
			.create_view(&wgpu::TextureViewDescriptor::default())
	}

	#[instrument(name = "Renderer::resize", skip_all)]
	pub fn resize(&mut self, new_width: u32, new_height: u32) {
		self.surface_config.width = new_width.max(1);
		self.surface_config.height = new_height.max(1);

		if new_width > 0 && new_height > 0 {
			self.surface.configure(&self.device, &self.surface_config);
			self.framebuffer = Self::create_framebuffer(&self.device, &self.surface_config, self.msaa_samples);
		}
	}

	#[instrument(name = "Renderer::prepare", skip_all)]
	pub fn prepare(&mut self, painter: &Painter, viewport: &ui::Viewport, text_atlas: &mut ui::TextAtlas) {
		let vertex_bytes = bytemuck::cast_slice(&painter.geometry.vertices);
		let index_bytes = bytemuck::cast_slice(&painter.geometry.indices);

		self.queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);
		self.queue.write_buffer(&self.index_buffer, 0, index_bytes);

		self.vertex_bytes = vertex_bytes.len() as u64;
		self.index_bytes = index_bytes.len() as u64;
		
		let [basis_x, basis_y, translation] = viewport.view_to_clip().columns();

		self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[
			Globals {
				row_x: [ basis_x.x, basis_y.x, translation.x, 0.0],
				row_y: [ basis_x.y, basis_y.y, translation.y, 0.0],
			}
		]));

		for ui::GlyphUpdate{image, dst_pos} in text_atlas.glyph_updates.drain(..) {
			let placement = image.placement;

			let mut data = image.data;

			if image.content == cosmic_text::SwashContent::Mask {
				data = data.into_iter()
					// TODO(pat.m): premultiply?
					.flat_map(|alpha| [255, 255, 255, alpha])
					.collect();
			}

			let image_copy = wgpu::ImageCopyTexture {
				texture: &self.text_atlas_texture,
				origin: wgpu::Origin3d {
					x: dst_pos.x as u32,
					y: dst_pos.y as u32,
					z: 0,
				},

				mip_level: 0,
				aspect: wgpu::TextureAspect::All,
			};

			let data_layout = wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(placement.width * 4),
				rows_per_image: None,
			};

			let size = wgpu::Extent3d {
				width: placement.width,
				height: placement.height,
				depth_or_array_layers: 1,
			};

			self.queue.write_texture(image_copy, &data, data_layout, size);
		}
	}

	#[instrument(name = "Renderer::present", skip_all)]
	pub fn present(&mut self) {
		if self.surface_config.width <= 0 || self.surface_config.height <= 0 {
			return;
		}

		let current_frame_surface_texture = match self.surface.get_current_texture() {
			Ok(frame) => frame,
			Err(_) => {
				self.resize(self.surface_config.width, self.surface_config.height);
				self.surface.get_current_texture()
					.expect("Failed to acquire next frame texture")
			}
		};

		let current_frame_view = current_frame_surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(
				if self.msaa_samples == 1 {
					wgpu::RenderPassColorAttachment {
						view: &current_frame_view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
							store: wgpu::StoreOp::Store,
						},
					}
				} else {
					wgpu::RenderPassColorAttachment {
						view: &self.framebuffer,
						resolve_target: Some(&current_frame_view),
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
							store: wgpu::StoreOp::Store,
						},
					}
				}
			)],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		if self.index_bytes > 0 {
			pass.set_pipeline(&self.vector_render_pipeline);
			pass.set_bind_group(0, &self.vector_bind_group, &[]);
			pass.set_index_buffer(self.index_buffer.slice(0..self.index_bytes), wgpu::IndexFormat::Uint32);
			pass.set_vertex_buffer(0, self.vertex_buffer.slice(0..self.vertex_bytes));

			let num_elements = (self.index_bytes/4) as u32;
			pass.draw_indexed(0..num_elements, 0, 0..1);
		}

		drop(pass);

		self.queue.submit(Some(encoder.finish()));
		current_frame_surface_texture.present();
	}
}





#[repr(C)]
#[derive(Copy, Clone)]
struct Globals {
	row_x: [f32; 4],
	row_y: [f32; 4],
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}



#[repr(C)]
#[derive(Copy, Clone)]
pub struct Vertex {
	pub pos: [f32; 2],
	pub color: [f32; 4],
	pub uv: [f32; 2],

	// min x, max x, min y, max y
	pub clip_rect: [u16; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
