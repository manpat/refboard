use crate::prelude::*;
use super::{WidgetId};

use std::any::TypeId;
use std::num::Wrapping;
use std::hash::{DefaultHasher, Hasher, Hash};


#[derive(Default, Debug)]
pub struct Hierarchy {
	pub root_node: HierarchyNode,
	pub nodes: HashMap<WidgetId, HierarchyNode>,
	pub epoch: Wrapping<u8>,
}

#[derive(Debug, Default)]
pub struct HierarchyNode {
	pub parent_id: Option<WidgetId>,
	pub children: Vec<WidgetId>,
	pub current_epoch_child_counter: usize,
	pub epoch: u8,
}

pub struct NodeUpdateResult {
	pub widget_id: WidgetId,
	pub status: NodeUpdateStatus,
}

pub enum NodeUpdateStatus {
	Added,
	Update,
}

pub enum WidgetIdFragment {
	TypedOrdered(TypeId),
}


impl Hierarchy {
	pub fn get_node(&self, widget_id: Option<WidgetId>) -> Option<&HierarchyNode> {
		match widget_id {
			None => Some(&self.root_node),
			Some(id) => self.nodes.get(&id),
		}
	}

	pub fn get_node_mut(&mut self, widget_id: Option<WidgetId>) -> Option<&mut HierarchyNode> {
		match widget_id {
			None => Some(&mut self.root_node),
			Some(id) => self.nodes.get_mut(&id),
		}
	}

	pub fn new_epoch(&mut self) {
		self.epoch += 1;
		self.root_node.update_epoch(self.epoch.0);
	}

	pub fn collect_stale_nodes(&mut self, mut f: impl FnMut(WidgetId)) {
		self.nodes.retain(|&widget_id, node| {
			if node.epoch != self.epoch.0 {
				f(widget_id);
				false
			} else {
				true
			}
		});
	}

	pub fn add_or_update(&mut self, id_fragment: impl Into<WidgetIdFragment>, parent_id: impl Into<Option<WidgetId>>) -> NodeUpdateResult {
		let parent_id = parent_id.into();
		let id_fragment = id_fragment.into();

		let current_epoch = self.epoch.0;

		let parent_node = self.get_node_mut(parent_id)
			.expect("Trying to add widget to invalid parent");

		assert!(parent_node.epoch == current_epoch, "Trying to add widget to stale parent");

		let child_index = parent_node.current_epoch_child_counter;
		parent_node.current_epoch_child_counter += 1;

		// Calculate widget id
		let mut hasher = DefaultHasher::new();
		parent_id.hash(&mut hasher);

		match id_fragment {
			WidgetIdFragment::TypedOrdered(type_id) => {
				type_id.hash(&mut hasher);
				hasher.write_usize(child_index);
			}
		}

		let widget_id = WidgetId(hasher.finish());

		// If the calculated id matches what in the same child position last epoch, then we only want to update
		if parent_node.children.get(child_index) == Some(&widget_id) {
			let child_node = self.nodes.get_mut(&widget_id).expect("Id match detected but missing node info");
			assert!(child_node.parent_id == parent_id, "Id match but parents mismatch - possibly a hash collision");
			assert!(child_node.epoch != current_epoch, "Node already updated - state may be inconsistent");

			child_node.update_epoch(current_epoch);

			return NodeUpdateResult {
				widget_id,
				status: NodeUpdateStatus::Update,
			}
		}

		// Otherwise we have to add/insert
		if let Some(child_id) = parent_node.children.get_mut(child_index) {
			*child_id = widget_id;
		} else {
			parent_node.children.push(widget_id);
		}

		let prev_value = self.nodes.insert(widget_id, HierarchyNode {
			parent_id,
			children: Vec::new(),
			current_epoch_child_counter: 0,
			epoch: current_epoch
		});

		assert!(prev_value.is_none(), "Widget already added to hierarchy!");

		NodeUpdateResult {
			widget_id,
			status: NodeUpdateStatus::Added,
		}
	}

	pub fn parent(&self, widget_id: WidgetId) -> Option<WidgetId> {
		self.nodes[&widget_id].parent_id
	}

	pub fn children(&self, widget_id: impl Into<Option<WidgetId>>) -> &[WidgetId] {
		match widget_id.into() {
			Some(widget_id) => self.nodes[&widget_id].children.as_slice(),
			None => self.root_node.children.as_slice(),
		}
	}

	pub fn visit_breadth_first<F>(&self, start: impl Into<Option<WidgetId>>, mut visit: F)
		where F: FnMut(WidgetId, &[WidgetId])
	{
		let start = start.into();
		let children = match start {
			Some(widget_id) => self.nodes[&widget_id].children.as_slice(),
			None => self.root_node.children.as_slice(),
		};

		// TODO(pat.m): reuse intermediate visit structures
		let mut visit_queue = VecDeque::new();

		visit_queue.extend(children.into_iter());

		while let Some(parent) = visit_queue.pop_front() {
			let children = self.nodes[&parent].children.as_slice();
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
				visit_stack.extend(self.root_node.children.iter().rev().map(|&id| (id, false)));
			}
		};

		while let Some((parent, children_visited)) = visit_stack.pop() {
			if children_visited {
				visit(parent);
				continue
			}

			let children = self.nodes[&parent].children.as_slice();
			if children.is_empty() {
				visit(parent);
				continue
			}

			visit_stack.push((parent, true));
			visit_stack.extend(children.iter().rev().map(|&id| (id, false)));
		}
	}
}



impl HierarchyNode {
	fn update_epoch(&mut self, epoch: u8) {
		self.epoch = epoch;
		self.current_epoch_child_counter = 0;
	}
}