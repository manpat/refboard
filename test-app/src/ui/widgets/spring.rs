use crate::ui::*;


#[derive(Debug)]
pub struct Spring(pub Axis);

impl Widget for Spring {
	fn configure(&self, ctx: ConfigureContext<'_>) {
		ctx.constraints.size_policy_mut(self.0).set_default(SizingBehaviour::FLEXIBLE);
		ctx.constraints.size_policy_mut(self.0.opposite()).set_default(SizingBehaviour::FIXED);
		ctx.constraints.self_alignment.set_default(Align::Middle);

		*ctx.input |= InputBehaviour::TRANSPARENT;
	}
}

impl Ui<'_> {
	pub fn spring(&self, axis: Axis) -> WidgetRef<'_, Spring> {
		// TODO(pat.m): can I derive Axis from context?
		self.add_widget(Spring(axis))
	}
}
