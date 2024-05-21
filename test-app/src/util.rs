

use std::any::{Any, type_name};


pub trait AsAny : Any {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T> AsAny for T
	where T: Any
{
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
}


// Separate from the above as Any has a 'static bound which makes things weird?
pub trait HasTypeName {
    fn type_name(&self) -> &'static str;
}

impl<T> HasTypeName for T
	where T: ?Sized
{
	fn type_name(&self) -> &'static str {
		type_name::<T>()
	}
}