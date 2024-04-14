use winit::{
	event::{Event, WindowEvent, StartCause},
	event_loop::{EventLoop, ControlFlow},
	window::{WindowBuilder, WindowLevel},
};

use std::sync::Arc;

use wgpu::util::DeviceExt;

use lyon::tessellation::*;
use lyon::tessellation::geometry_builder::*;

use common::*;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
	env_logger::init();

	// use winit::platform::windows::WindowBuilderExtWindows;

	let event_loop = EventLoop::new()?;
	let window = WindowBuilder::new()
		.with_title("refboard")
		.with_resizable(true)
		// .with_transparent(true) // Doesn't work
		// .with_decorations(false)
		.with_window_level(WindowLevel::AlwaysOnTop)
		.with_visible(false)
		.build(&event_loop)?;

	let window = Arc::new(window);

	// #[cfg(windows)]
	// unsafe {
	// 	use raw_window_handle::{HasWindowHandle, RawWindowHandle};
	// 	use windows::{Win32::{UI::WindowsAndMessaging::*, Foundation::*}};

	// 	let RawWindowHandle::Win32(handle) = window.window_handle()?.as_raw() else { anyhow::bail!("Failed to get window handle"); };
	// 	let hwnd = HWND(handle.hwnd.get());

	// 	SetWindowLongPtrA(hwnd,
	// 		GWL_EXSTYLE,
	// 		GetWindowLongA(hwnd, GWL_EXSTYLE) as isize | WS_EX_LAYERED.0 as isize);

	// 	SetLayeredWindowAttributes(hwnd, COLORREF(0), ((255 * 30) / 100) as u8, LWA_ALPHA)?;
	// }

	println!("window created");

	let mut renderer = Renderer::start(window.clone()).await?;


	// Create tessellated shape
	let mut painter = Painter::new();

	use lyon::path::math::{point};

	let mut builder = Path::builder();
	builder.begin(point(-7.0, 0.0));
	builder.line_to(point(-7.0, 7.0));
	builder.line_to(point(0.0, 7.0));
	builder.quadratic_bezier_to(point(7.0, 7.0), point(7.0, 0.0));
	builder.end(false);

	let path = builder.build();

	painter.fill_path(&path, [0.3, 0.2, 0.9]);
	painter.stroke_path(&path, [0.3, 1.0, 0.6]);

	painter.fill_circle(Vec2::zero(), 5.0, [1.0, 0.5, 1.0]);
	painter.circle(Vec2::zero(), 5.0, [0.3, 0.1, 0.35]);

	painter.rect(Aabb2::around_point(Vec2::zero(), Vec2::new(9.0, 9.0)), [1.0, 1.0, 1.0]);

	event_loop.run(move |event, target| {
		// target.set_control_flow(ControlFlow::Wait);
		target.set_control_flow(ControlFlow::Poll);

		// Initial present/show window
		if let Event::NewEvents(StartCause::Init) = event {
			renderer.prepare(&painter);

			window.pre_present_notify();
			renderer.present();

			window.set_visible(true);
		}

		if let Event::WindowEvent { window_id, event } = event {
			match event {
				WindowEvent::RedrawRequested => {
					renderer.prepare(&painter);

					window.pre_present_notify();
					renderer.present();
				}
				
				WindowEvent::Resized(new_size) => {
					renderer.resize(new_size.width, new_size.height);
					renderer.prepare(&painter);
					renderer.present();
				}

				WindowEvent::CloseRequested => {
					target.exit();
				}

				_ => {}
			}
		}
	})
	.map_err(Into::into)
}

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
	pos: [f32; 2],
	color: [f32; 4],
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

struct VertexConstructor {
	// transform: usvg::Transform,
	// color: epaint::Color32,
	color: [f32; 4],

}

impl FillVertexConstructor<Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
		let pos = vertex.position().to_array();
		Vertex {
			pos,
			color: self.color,
		}
	}
}

impl StrokeVertexConstructor<Vertex> for VertexConstructor {
	fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
		let pos = vertex.position().to_array();
		Vertex {
			pos,
			color: self.color,
		}
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



pub struct Painter {
	geometry: VertexBuffers<Vertex, u32>,
	fill_tess: FillTessellator,
	stroke_tess: StrokeTessellator,

	fill_options: FillOptions,
	stroke_options: StrokeOptions,
}

impl Painter {
	pub fn new() -> Painter {
		Painter {
			geometry: VertexBuffers::new(),
			fill_tess: FillTessellator::new(),
			stroke_tess: StrokeTessellator::new(),

			fill_options: FillOptions::tolerance(0.02).with_fill_rule(FillRule::NonZero),
			stroke_options: StrokeOptions::DEFAULT,
		}
	}

	pub fn clear(&mut self) {
		self.geometry.vertices.clear();
		self.geometry.indices.clear();
	}

	fn geo_builder<'g>(geo: &'g mut VertexBuffers<Vertex, u32>, color: impl Into<Color>) -> (impl StrokeGeometryBuilder + FillGeometryBuilder + 'g) {
		let color = color.into().to_array();
		BuffersBuilder::new(geo, VertexConstructor { color }).with_inverted_winding()
	}
}

use lyon::math::{Point, Box2D};
use lyon::path::{Path, PathSlice};
use lyon::tessellation::*;
use lyon::tessellation::geometry_builder::*;

fn to_point(Vec2{x,y}: Vec2) -> Point {
	Point::new(x, y)
}

fn to_box(Aabb2{min, max}: Aabb2) -> Box2D {
	Box2D::new(to_point(min), to_point(max))
}

impl Painter {
	pub fn circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.stroke_tess.tessellate_circle(pos, r, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_circle(&mut self, pos: impl Into<Vec2>, r: f32, color: impl Into<Color>) {
		let pos = to_point(pos.into());
		self.fill_tess.tessellate_circle(pos, r, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.stroke_tess.tessellate_rectangle(&rect, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_rect(&mut self, rect: impl Into<Aabb2>, color: impl Into<Color>) {
		let rect = to_box(rect.into());
		self.fill_tess.tessellate_rectangle(&rect, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn stroke_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.stroke_tess.tessellate_path(path, &self.stroke_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}

	pub fn fill_path(&mut self, path: &Path, color: impl Into<Color>) {
		self.fill_tess.tessellate_path(path, &self.fill_options, &mut Self::geo_builder(&mut self.geometry, color)).unwrap();
	}
}




pub struct Renderer {
	device: wgpu::Device,
	queue: wgpu::Queue,
	surface: wgpu::Surface<'static>,
	surface_config: wgpu::SurfaceConfiguration,

	globals_buffer: wgpu::Buffer,
	vector_bind_group: wgpu::BindGroup,
	vector_render_pipeline: wgpu::RenderPipeline,

	vertex_buffer: wgpu::Buffer,
	index_buffer: wgpu::Buffer,

	vertex_bytes: u64,
	index_bytes: u64,
}

impl Renderer {
	pub async fn start(window: Arc<winit::window::Window>) -> anyhow::Result<Renderer> {
		let size = window.inner_size();

		// create an instance
		let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

		// create an surface
		let surface = instance.create_surface(window)?;

		println!("surface created");

		// create an adapter
		let Some(adapter) = instance.request_adapter(&wgpu::RequestAdapterOptions {
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
				required_features: wgpu::Features::default(),
				required_limits: wgpu::Limits::default(),
			},
			None,
		)
		.await?;

		println!("device created");

		let mut surface_config = surface.get_default_config(&adapter, size.width, size.height).ok_or_else(|| anyhow::format_err!("Failed to get surface config"))?;
		surface_config.present_mode = wgpu::PresentMode::AutoVsync;
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
			],
		});

		let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: None,
			bind_group_layouts: &[&bind_group_layout],
			push_constant_ranges: &[],
		});

		let swapchain_capabilities = surface.get_capabilities(&adapter);
		let swapchain_format = swapchain_capabilities.formats[0];

		let vector_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: None,
			layout: Some(&pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[wgpu::VertexBufferLayout {
					array_stride: std::mem::size_of::<Vertex>() as u64,
					step_mode: wgpu::VertexStepMode::Vertex,
					attributes: &[
						wgpu::VertexAttribute {
							offset: 0,
							format: wgpu::VertexFormat::Float32x2,
							shader_location: 0,
						},
						wgpu::VertexAttribute {
							offset: 8,
							format: wgpu::VertexFormat::Float32x4,
							shader_location: 1,
						},
					],
				}],
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[Some(wgpu::ColorTargetState {
					format: swapchain_format,
					write_mask: wgpu::ColorWrites::all(),
					blend: Some(wgpu::BlendState::ALPHA_BLENDING),
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
			multisample: wgpu::MultisampleState::default(),
			multiview: None,
		});

		let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: 1<<20,
			usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: None,
			size: 1<<20,
			usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let globals_buffer = device.create_buffer(&wgpu::BufferDescriptor {
			label: Some("Globals ubo"),
			size: globals_buffer_byte_size,
			usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
			mapped_at_creation: false,
		});

		let vector_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: Some("Bind group"),
			layout: &bind_group_layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::Buffer(globals_buffer.as_entire_buffer_binding()),
				},
			],
		});

		Ok(Renderer {
			device,
			queue,
			surface,
			surface_config,

			globals_buffer,
			vector_bind_group,
			vector_render_pipeline,

			vertex_buffer,
			index_buffer,

			vertex_bytes: 0,
			index_bytes: 0,
		})
	}

	pub fn resize(&mut self, new_width: u32, new_height: u32) {
		self.surface_config.width = new_width;
		self.surface_config.height = new_height;

		if new_width > 0 && new_height > 0 {
			self.surface.configure(&self.device, &self.surface_config);
		}
	}

	pub fn prepare(&mut self, painter: &Painter) {
		let vertex_bytes = bytemuck::cast_slice(&painter.geometry.vertices);
		let index_bytes = bytemuck::cast_slice(&painter.geometry.indices);

		self.queue.write_buffer(&self.vertex_buffer, 0, vertex_bytes);
		self.queue.write_buffer(&self.index_buffer, 0, index_bytes);

		self.vertex_bytes = vertex_bytes.len() as u64;
		self.index_bytes = index_bytes.len() as u64;
		
		let surface_width = self.surface_config.width as f32;
		let surface_height = self.surface_config.height as f32;

		let aspect = surface_width / surface_height;
		let scale = 0.04;
		self.queue.write_buffer(&self.globals_buffer, 0, bytemuck::cast_slice(&[
			Globals {
				row_x: [ scale/aspect, 0.0, 0.0,  0.0],
				row_y: [         0.0, scale, 0.0, 0.0],
			}
		]));
	}

	pub fn present(&self) {
		if self.surface_config.width <= 0 || self.surface_config.height <= 0 {
			return;
		}

		let current_frame_surface_texture = self.surface.get_current_texture().unwrap();
		let current_frame_view = current_frame_surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

		let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			label: None,
			color_attachments: &[Some(
				wgpu::RenderPassColorAttachment {
					view: &current_frame_view,
					resolve_target: None,
					ops: wgpu::Operations {
						load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
						store: wgpu::StoreOp::Store,
					},
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