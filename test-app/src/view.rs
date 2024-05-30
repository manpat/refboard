use crate::prelude::*;

pub struct View {
	// item view
	// gizmo overlay
	// menus
}

impl View {
	pub fn new() -> View {
		View {
		}
	}

	pub fn build(&mut self, ui: &ui::Ui<'_>, app: &app::App) {
		ui.push_layout(
			ui.add_widget(ui::FrameWidget::vertical())
				.with_constraints(|c| c.set_size_policy(ui::SizingBehaviour::FLEXIBLE))
		);

		ui.text("Hello, Rust! 🦀🐄🦐🅱\nI'm emoting العربية ");

		ui.with_parent_widget(ui::FrameWidget::horizontal().with_color(Color::white()), || {
			ui.text("White bg color🦀🐄🦐🅱")
				.with_widget(|w, _| w.color = Color::black());
		})
		.with_constraints(|c| c.self_alignment.set(ui::Align::End));

		ui.text("Now I'm centre aligned")
			.with_constraints(|c| c.self_alignment.set(ui::Align::Middle));

		let add_widget = |idx| {
			let frame = ui.add_widget(ui::FrameWidget::horizontal().with_color(Color::green().with_alpha(0.2)));

			if frame.is_hovered() {
				frame.widget().background_color = Color::green();
			}

			if frame.is_clicked() {
				println!("CLICK! {idx}");
				app.dummy.set(app.dummy.get() + 1);
				app.hack_changed.set(true);
			}

			frame
		};

		ui.with_parent_widget(ui::FrameWidget::horizontal(), || {
			for _ in 0..app.dummy.get() {
				add_widget(5).with_constraints(|c| c.set_size((50.0, 50.0)));
			}
		})
		.with_constraints(|c| c.max_width.set(280.0));

		let frame = ui.add_widget(ui::FrameWidget::horizontal())
			.with_constraints(|c| c.margin.set(8.0));

		if frame.is_hovered() {
			frame.widget().background_color = Color::grey_a(0.5, 0.1);
		}

		if frame.is_clicked() {
			app.dummy.set(app.dummy.get().saturating_sub(1));
			app.hack_changed.set(true);
		}

		ui.with_parent(frame, || {
			add_widget(0).constraints().set_size((20.0, 20.0));
			add_widget(1).constraints().set_size((50.0, 50.0));
			add_widget(2).constraints().set_size((100.0, 50.0));
			add_widget(3).constraints().set_size((50.0, 100.0));
		});


		ui.with_horizontal_layout(|| {
			#[derive(Default, Debug)]
			struct Stateful;

			#[derive(Default)]
			struct StatefulState {
				active: bool,
				hovered: bool,
			}

			impl ui::Widget for Stateful {
				fn draw(&self, ctx: ui::DrawContext<'_>) {
					let state = ctx.state.get_or_default::<StatefulState>();

					let color = match (state.active, state.hovered) {
						(true, false) => Color::green(),
						(true, true) => Color::light_green(),

						(false, false) => Color::red(),
						(false, true) => Color::light_red(),
					};

					ui::FrameWidget::horizontal()
						.with_color(color)
						.draw(ctx);
				}
			}

			#[allow(non_local_definitions)]
			impl ui::WidgetRef<'_, Stateful> {
				fn is_active(&self) -> bool {
					self.state_as::<StatefulState>().active
				}
			}

			let widget = ui.add_widget(Stateful)
				.with_constraints(|c| c.set_size((32.0, 32.0)));

			widget.state_as::<StatefulState>().hovered = widget.is_hovered();

			if widget.is_clicked() {
				let mut state = widget.state_as::<StatefulState>();
				state.active = !state.active;
			}

			ui.add_widget(())
				.with_constraints(|c| match widget.is_active() {
					true => c.horizontal_size_policy.set(ui::SizingBehaviour::FLEXIBLE),
					false => c.set_width(40.0),
				});
		})
		.with_constraints(|c| c.horizontal_size_policy.set(ui::SizingBehaviour::FLEXIBLE));


		ui.spring(ui::Axis::Vertical);

		ui.with_horizontal_layout(|| {
			if ui.button().is_clicked() {
				app.dummy.set(app.dummy.get() + 1);
				app.hack_changed.set(true);
			}

			if ui.button().is_clicked() {
				app.dummy.set(app.dummy.get().saturating_sub(1));
				app.hack_changed.set(true);
			}
		});

		ui.add_widget(ui::DrawFnWidget(|ctx| {
			// TODO(pat.m): yuck
			ctx.painter.set_clip_rect([0, 65535, 0, 65535]);

			// Cursor
			if let Some(cursor_pos) = ctx.input.cursor_pos_view {
				ctx.painter.set_color([1.0, 0.0, 0.0]);
				ctx.painter.circle(cursor_pos, 3.0);
			}
		}));

		ui.pop_layout();
	}
}




