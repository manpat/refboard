use std::any::{TypeId, Any};

#[derive(Debug, Default)]
pub struct StateBox(Option<Box<dyn Any>>);

impl StateBox {
	pub fn empty() -> Self {
		StateBox(None)
	}

	pub fn new<T>(value: T) -> Self
		where T: 'static
	{
		StateBox(Some(Box::new(value)))
	}

	pub fn has<T: 'static>(&self) -> bool {
		match &self.0 {
			Some(value) => (*value).is::<T>(),
			None => false,
		}
	}

	pub fn set<T>(&mut self, value: T)
		where T: 'static
	{
		self.0 = Some(Box::new(value));
	}

	pub fn get_or_default<T>(&mut self) -> &mut T
		where T: Default + 'static
	{
		// If we have a value but the type is wrong, reset it to default
		if !self.has::<T>() {
			self.0 = Some(Box::new(T::default()));
		}

		// SAFETY: We're calling set above, so we know that the Option is populated and with what type.
		unsafe {
			// TODO(pat.m): this could use downcast_mut_unchecked once stable
			self.0.as_mut()
				.and_then(|value| value.downcast_mut::<T>())
				.unwrap_unchecked()
		}
	}

	pub fn get<T>(&mut self) -> &mut T
		where T: 'static
	{
		assert!(self.has::<T>());

		unsafe {
			// TODO(pat.m): this could use downcast_mut_unchecked once stable
			self.0.as_mut()
				.and_then(|value| value.downcast_mut::<T>())
				.unwrap_unchecked()
		}
	}
}