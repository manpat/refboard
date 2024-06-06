use std::any::Any;

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

	pub fn get_or_else<T, F>(&mut self, create: F) -> &mut T
		where T: 'static
			, F: FnOnce() -> T
	{
		// If we have no value or a value with incorrect type, create a fresh one
		if !self.has::<T>() {
			self.0 = Some(Box::new(create()));
		}

		// SAFETY: We're set self.0 above, so we know that the Option is populated and with what type.
		unsafe {
			// TODO(pat.m): this could use downcast_mut_unchecked once stable
			self.0.as_mut()
				.and_then(|value| value.downcast_mut::<T>())
				.unwrap_unchecked()
		}
	}

	pub fn get_or_default<T>(&mut self) -> &mut T
		where T: Default + 'static
	{
		self.get_or_else(Default::default)
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