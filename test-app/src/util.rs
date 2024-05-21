

use std::any::Any;


pub trait AsAny : Any {
	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;
	// fn type_name(&self) -> &'static str;
}

impl<T> AsAny for T
	where T: Any
{
	fn as_any(&self) -> &dyn Any { self }
	fn as_any_mut(&mut self) -> &mut dyn Any { self }
	// fn type_name(&'_ self) -> &'static str {
	// 	Any::type_name(self)
	// }
}