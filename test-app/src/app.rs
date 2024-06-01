use crate::prelude::*;

#[derive(Default)]
pub struct App {
	pub persistence: AppPersistence,
	pub board: Board,
	pub undo_stack: UndoStack,

	pub dummy: Cell<u32>,
	pub hack_changed: Cell<bool>,

	pub command_queue: Vec<AppCommand>,
}

impl App {
	pub fn apply_changes(&mut self) {
		// TODO(pat.m): process command queue, applying changes to Board and updating undo_stack if necessary
		// TODO(pat.m): notify view of anything it needs to know?
	}
}


#[derive(Default)]
pub struct AppPersistence {
	// window geometry
	// last open board
	// theme
	// keymap
	// general settings
}

#[derive(Default)]
pub struct UndoStack {}


slotmap::new_key_type! {
	pub struct ItemKey;
	pub struct ImageKey;
}


#[derive(Default, Debug)]
pub struct Board {
	pub items: slotmap::SlotMap<ItemKey, Item>,

	// TODO(pat.m): maybe this shouldn't be in a board?
	pub images: slotmap::SlotMap<ImageKey, Image>,
}

#[derive(Debug)]
pub struct Item {
	pub transform: Mat2x3,
	pub data: ItemData,
}


#[derive(Debug)]
pub enum ItemData {
	Text {
		content: String,
	},

	Scribble,
	Image {
		key: ImageKey
		// geometry - cropping/mesh
	},
}

#[derive(Debug)]
pub struct Image {
	// origin metadata
	// compressed contents?
	// dimensions
}


pub enum AppCommand {
	NewBoard,
	SaveBoard,
	LoadBoard,
	Quit,

	AddItem,
	DeleteItem,
	TransformItem,

	EditText,
	EditScribble,
	EditImage,
}
