use cty::c_int;

use crate::get_error;
use ndless::alloc::string::String;
use ndless::alloc::vec::Vec;
pub mod ll {
	#![allow(non_camel_case_types)]

	use cty::{c_int, c_void, int16_t, uint16_t, uint8_t};

	use crate::Rect;

	pub static SDL_DISABLE: c_int = 0;
	pub static SDL_ENABLE: c_int = 1;
	pub static SDL_QUERY: c_int = -1;

	pub type WMcursor = c_void;

	#[repr(C)]
	#[derive(Copy, Clone)]
	pub struct SDL_Cursor {
		pub area: Rect,
		pub hot_x: int16_t,
		pub hot_y: int16_t,
		pub data: *mut uint8_t,
		pub mask: *mut uint8_t,
		pub save: [*mut uint8_t; 2],
		pub wm_cursor: *mut WMcursor,
	}

	extern "C" {
		pub fn SDL_ShowCursor(toggle: c_int) -> c_int;
		pub fn SDL_CreateCursor(
			data: *mut uint8_t,
			mask: *mut uint8_t,
			w: c_int,
			h: c_int,
			hot_x: c_int,
			hot_y: c_int,
		) -> *mut SDL_Cursor;
		pub fn SDL_SetCursor(cursor: *mut SDL_Cursor);
		pub fn SDL_GetCursor() -> *mut SDL_Cursor;
		pub fn SDL_FreeCursor(cursor: *mut SDL_Cursor);
		pub fn SDL_WarpMouse(x: uint16_t, y: uint16_t);
	}
}

pub fn warp_mouse(x: u16, y: u16) {
	unsafe {
		ll::SDL_WarpMouse(x, y);
	}
}

#[derive(PartialEq)]
pub struct Cursor {
	pub raw: *mut ll::SDL_Cursor,
	pub owned: bool,
}

fn wrap_cursor(raw: *mut ll::SDL_Cursor, owned: bool) -> Cursor {
	Cursor { raw, owned }
}

impl Cursor {
	pub fn new(
		data: &[u8],
		mask: &[u8],
		w: isize,
		h: isize,
		hot_x: isize,
		hot_y: isize,
	) -> Result<Cursor, String> {
		let mut data = data.to_vec();
		let mut mask = mask.to_vec();
		unsafe {
			let raw = ll::SDL_CreateCursor(
				data.as_mut_ptr(),
				mask.as_mut_ptr(),
				w as c_int,
				h as c_int,
				hot_x as c_int,
				hot_y as c_int,
			);

			if raw.is_null() {
				Err(get_error())
			} else {
				Ok(wrap_cursor(raw, true))
			}
		}
	}
}

impl Drop for Cursor {
	fn drop(&mut self) {
		unsafe {
			if self.owned {
				ll::SDL_FreeCursor(self.raw);
			}
		}
	}
}

pub fn set_cursor(cursor: &Cursor) {
	unsafe {
		ll::SDL_SetCursor(cursor.raw);
	}
}

pub fn get_cursor() -> Cursor {
	unsafe { wrap_cursor(ll::SDL_GetCursor(), false) }
}

pub fn set_cursor_visible(visible: bool) {
	unsafe {
		ll::SDL_ShowCursor(visible as c_int);
	}
}

pub fn toggle_cursor_visible() {
	unsafe {
		if ll::SDL_ShowCursor(ll::SDL_QUERY) == ll::SDL_ENABLE {
			ll::SDL_ShowCursor(ll::SDL_DISABLE);
		} else {
			ll::SDL_ShowCursor(ll::SDL_ENABLE);
		}
	}
}

pub fn is_cursor_visible() -> bool {
	unsafe { ll::SDL_ShowCursor(ll::SDL_QUERY) == ll::SDL_ENABLE }
}
