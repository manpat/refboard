use winit::{
	event::{Event, WindowEvent, StartCause},
	event_loop::{EventLoop, ControlFlow},
	window::{WindowBuilder, WindowLevel},
};

use std::sync::Arc;


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

	// create an instance
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
		// backends: wgpu::Backends::all().difference(wgpu::Backends::VULKAN),
		// backends: wgpu::Backends::GL,
		.. wgpu::InstanceDescriptor::default()
	});

	// create an surface
	let surface = instance.create_surface(window.clone())?;

	println!("surface created");

	// for adapter in instance.enumerate_adapters(wgpu::Backends::all()) {
	// 	dbg!(adapter.get_info(), surface.get_capabilities(&adapter).alpha_modes);
	// }

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

	// // create a device and a queue
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

	let size = window.inner_size();
	let mut config = surface.get_default_config(&adapter, size.width, size.height).ok_or_else(|| anyhow::format_err!("Failed to get surface config"))?;
	// config.alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
	surface.configure(&device, &config);

	println!("surface configured");

	event_loop.run(move |event, target| {
		target.set_control_flow(ControlFlow::Poll);

		// Initial present/show window
		if let Event::NewEvents(StartCause::Init) = event {
			let current_frame_surface_texture = surface.get_current_texture().unwrap();
			let current_frame_view = current_frame_surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

			let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

			{
				let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: None,
					color_attachments: &[Some(
						wgpu::RenderPassColorAttachment {
							view: &current_frame_view,
							resolve_target: None,
							ops: wgpu::Operations {
								load: wgpu::LoadOp::Clear(wgpu::Color::RED),
								store: wgpu::StoreOp::Store,
							},
						}
					)],
					depth_stencil_attachment: None,
					timestamp_writes: None,
					occlusion_query_set: None,
				});
			}

			queue.submit(Some(encoder.finish()));
			current_frame_surface_texture.present();

			window.set_visible(true);
		}

		if let Event::WindowEvent { window_id, event } = event {
			match event {
				WindowEvent::RedrawRequested => {
					if config.width <= 0 || config.height <= 0 {
						return
					}

					let current_frame_surface_texture = surface.get_current_texture().unwrap();
					let current_frame_view = current_frame_surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default());

					let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

					{
						let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: None,
							color_attachments: &[Some(
								wgpu::RenderPassColorAttachment {
									view: &current_frame_view,
									resolve_target: None,
									ops: wgpu::Operations {
										load: wgpu::LoadOp::Clear(wgpu::Color::RED),
										store: wgpu::StoreOp::Store,
									},
								}
							)],
							depth_stencil_attachment: None,
							timestamp_writes: None,
							occlusion_query_set: None,
						});
					}

					queue.submit(Some(encoder.finish()));
					current_frame_surface_texture.present();
				}
				
				WindowEvent::Resized(new_size) => {
					config.width = new_size.width;
					config.height = new_size.height;

					if config.width > 0 && config.height > 0 {
						surface.configure(&device, &config);
						window.request_redraw();
					}
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
