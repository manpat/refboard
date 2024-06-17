use crate::prelude::*;

// use wgpu::util::DeviceExt;
use std::sync::Arc;

pub struct Renderer {
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,

	framebuffer: wgpu::TextureView,

	// TODO(pat.m): not shared bc of msaa_samples
	vector_render_pipeline: wgpu::RenderPipeline,


	vertex_bytes: u64,
	index_bytes: u64,

	msaa_samples: u32,
}

impl Renderer {
	#[instrument(skip_all)]
	pub fn new(core: &GraphicsCore, shared_resources: &SharedResources, window: Arc<winit::window::Window>) -> anyhow::Result<Renderer> {
		// create an surface
		let size = window.inner_size();
		let surface = core.instance.create_surface(window)?;

		log::trace!("surface created");

		let mut surface_config = surface.get_default_config(&core.adapter, size.width, size.height).ok_or_else(|| anyhow::format_err!("Failed to get surface config"))?;
		surface_config.present_mode = wgpu::PresentMode::AutoNoVsync;
		surface.configure(&core.device, &surface_config);

		log::trace!("surface configured");



		let swapchain_capabilities = surface.get_capabilities(&core.adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		let supported_sample_counts = core.adapter.get_texture_format_features(swapchain_format).flags.supported_sample_counts();
		let msaa_samples = supported_sample_counts.into_iter().max().unwrap_or(1);

		log::info!("Using MSAA x{msaa_samples}");

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

		let vector_render_pipeline = core.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&shared_resources.pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shared_resources.shader_module,
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
				module: &shared_resources.shader_module,
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

		let framebuffer = Self::create_framebuffer(&core.device, &surface_config, msaa_samples);

		log::trace!("renderer init completed");

		Ok(Renderer {
			surface,
			surface_config,

			vector_render_pipeline,
			framebuffer,

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
	pub fn resize(&mut self, core: &GraphicsCore, new_width: u32, new_height: u32) {
		self.surface_config.width = new_width.max(1);
		self.surface_config.height = new_height.max(1);

		if new_width > 0 && new_height > 0 {
			self.surface.configure(&core.device, &self.surface_config);
			self.framebuffer = Self::create_framebuffer(&core.device, &self.surface_config, self.msaa_samples);
		}
	}

	#[instrument(name = "Renderer::prepare", skip_all)]
	pub fn prepare(&mut self, core: &GraphicsCore, shared_resources: &SharedResources,
		painter: &Painter, viewport: &ui::Viewport, text_atlas: &mut ui::TextAtlas)
	{
		let vertex_bytes = bytemuck::cast_slice(&painter.geometry.vertices);
		let index_bytes = bytemuck::cast_slice(&painter.geometry.indices);

		core.queue.write_buffer(&shared_resources.vertex_buffer, 0, vertex_bytes);
		core.queue.write_buffer(&shared_resources.index_buffer, 0, index_bytes);

		self.vertex_bytes = vertex_bytes.len() as u64;
		self.index_bytes = index_bytes.len() as u64;
		
		let [basis_x, basis_y, translation] = viewport.view_to_clip().columns();

		core.queue.write_buffer(&shared_resources.globals_buffer, 0, bytemuck::cast_slice(&[
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
					.flat_map(|alpha| [255, 255, 255, alpha])
					.collect();
			}

			let image_copy = wgpu::ImageCopyTexture {
				texture: &shared_resources.text_atlas_texture,
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

			core.queue.write_texture(image_copy, &data, data_layout, size);
		}
	}

	#[instrument(name = "Renderer::present", skip_all)]
	pub fn present(&mut self, core: &GraphicsCore, shared_resources: &SharedResources) {
		if self.surface_config.width <= 0 || self.surface_config.height <= 0 {
			return;
		}

		let current_frame_surface_texture = match self.surface.get_current_texture() {
			Ok(frame) => frame,
			Err(_) => {
				self.resize(core, self.surface_config.width, self.surface_config.height);
				self.surface.get_current_texture()
					.expect("Failed to acquire next frame texture")
			}
		};

		let current_frame_view = current_frame_surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = core.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

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
							store: wgpu::StoreOp::Discard,
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
			pass.set_bind_group(0, &shared_resources.vector_bind_group, &[]);
			pass.set_index_buffer(shared_resources.index_buffer.slice(0..self.index_bytes), wgpu::IndexFormat::Uint32);
			pass.set_vertex_buffer(0, shared_resources.vertex_buffer.slice(0..self.vertex_bytes));

			let num_elements = (self.index_bytes/4) as u32;
			pass.draw_indexed(0..num_elements, 0, 0..1);
		}

		drop(pass);

		core.queue.submit(Some(encoder.finish()));
		current_frame_surface_texture.present();
	}
}





#[repr(C)]
#[derive(Copy, Clone)]
struct Globals {
	row_x: [f32; 4],
	row_y: [f32; 4],
}

impl Globals {
	const SIZE: u64 = std::mem::size_of::<Globals>() as u64;
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





pub struct SharedResources {
	shader_module: wgpu::ShaderModule,
	text_atlas_texture: wgpu::Texture,

	globals_buffer: wgpu::Buffer,
	vector_bind_group: wgpu::BindGroup,
	pipeline_layout: wgpu::PipelineLayout,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,
}

impl SharedResources {
	pub fn new(core: &GraphicsCore) -> anyhow::Result<SharedResources> {
		let shader_module = core.device.create_shader_module(wgpu::include_wgsl!("shaders.wgsl"));
		
		let bind_group_layout = core.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("Bind group layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::VERTEX,
					ty: wgpu::BindingType::Buffer {
						ty: wgpu::BufferBindingType::Uniform,
						has_dynamic_offset: false,
						min_binding_size: wgpu::BufferSize::new(Globals::SIZE),
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

		let pipeline_layout = core.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let vertex_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ui vertex buffer"),
			size: 8<<20,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let index_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("ui index buffer"),
			size: 8<<20,
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let globals_buffer = core.device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Globals ubo"),
			size: Globals::SIZE,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let text_atlas_texture = core.device.create_texture(&wgpu::TextureDescriptor {
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

		let atlas_sampler = core.device.create_sampler(&wgpu::SamplerDescriptor {
			label: Some("Text Sampler"),
			min_filter: wgpu::FilterMode::Linear,
			mag_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,

			.. Default::default()
		});

		let vector_bind_group = core.device.create_bind_group(&wgpu::BindGroupDescriptor {
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
			core.queue.write_texture(image_copy, &[255; 4 * 4], data_layout, size);
		}


		Ok(SharedResources {
			shader_module,
			text_atlas_texture,
			globals_buffer,
			vector_bind_group,
			pipeline_layout,
			vertex_buffer,
			index_buffer,
		})
	}
}


pub struct GraphicsCore {
	pub instance: wgpu::Instance,
	pub adapter: wgpu::Adapter,
	pub device: wgpu::Device,
	pub queue: wgpu::Queue,
}

impl GraphicsCore {
	pub async fn new() -> anyhow::Result<GraphicsCore> {
		// create an instance
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

		log::trace!("wgpu instance created");

		// create an adapter
		let Some(adapter) = instance.request_adapter(&wgpu::RequestAdapterOptions {
			// power_preference: wgpu::PowerPreference::HighPerformance,
			power_preference: wgpu::PowerPreference::LowPower,
			compatible_surface: None,
			force_fallback_adapter: false,
		}).await
			else {
				anyhow::bail!("Failed to request adapter")
			};

		log::trace!("adapter created");

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

		log::trace!("device created");

		Ok(GraphicsCore {
			instance,
			adapter,
			device,
			queue,
		})
	}
}