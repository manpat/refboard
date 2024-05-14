use crate::prelude::*;
use super::{WidgetId};


#[derive(Default, Debug)]
pub struct Hierarchy {
	pub info: HashMap<WidgetId, HierarchyNode>,
	pub root_nodes: Vec<WidgetId>,
}

#[derive(Debug)]
pub struct HierarchyNode {
	pub parent: Option<WidgetId>,
	pub children: Vec<WidgetId>,
}

impl Hierarchy {
	pub fn add(&mut self, widget_id: WidgetId, parent: impl Into<Option<WidgetId>>) {
		let parent = parent.into();

		if let Some(parent) = parent {
			self.info.get_mut(&parent).unwrap().children.push(widget_id);
		} else {
			self.root_nodes.push(widget_id);
		}

		let prev_value = self.info.insert(widget_id, HierarchyNode{parent, children: Vec::new()});
		assert!(prev_value.is_none(), "Widget already added to hierarchy!");
	}

	pub fn parent(&self, widget_id: WidgetId) -> Option<WidgetId> {
		self.info[&widget_id].parent
	}

	pub fn children(&self, widget_id: impl Into<Option<WidgetId>>) -> &[WidgetId] {
		match widget_id.into() {
			Some(widget_id) => self.info[&widget_id].children.as_slice(),
			None => self.root_nodes.as_slice(),
		}
	}

	pub fn visit_breadth_first<F>(&self, start: impl Into<Option<WidgetId>>, mut visit: F)
		where F: FnMut(WidgetId, &[WidgetId])
	{
		let start = start.into();
		let children = match start {
			Some(widget_id) => self.info[&widget_id].children.as_slice(),
			None => self.root_nodes.as_slice(),
		};

		// TODO(pat.m): reuse intermediate visit structures
		let mut visit_queue = VecDeque::new();

		visit_queue.extend(children.into_iter());

		while let Some(parent) = visit_queue.pop_front() {
			let children = self.info[&parent].children.as_slice();
			visit_queue.extend(children.iter().copied());

			visit(parent, children);
		}
	}

	/// Postorder traversal
	pub fn visit_leaves_first<F>(&self, start: impl Into<Option<WidgetId>>, mut visit: F)
		where F: FnMut(WidgetId)
	{
		// TODO(pat.m): reuse intermediate visit structures
		let mut visit_stack = Vec::new();

		let start = start.into();
		match start {
			Some(widget_id) => {
				visit_stack.push((widget_id, false));
			}

			None => {
				visit_stack.extend(self.root_nodes.iter().rev().map(|&id| (id, false)));
			}
		};

		while let Some((parent, children_visited)) = visit_stack.pop() {
			if children_visited {
				visit(parent);
				continue
			}

			let children = self.info[&parent].children.as_slice();
			if children.is_empty() {
				visit(parent);
				continue
			}

			visit_stack.push((parent, true));
			visit_stack.extend(children.iter().rev().map(|&id| (id, false)));
		}
	}
}

