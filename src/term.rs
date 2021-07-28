/*
 * The view handles what we see on the terminal.
 */

use crate::grid;

extern crate ncurses;
use ncurses::*;

use std::convert::TryInto;
use std::sync::atomic::{AtomicBool, Ordering};

static NCURSES_FLAG: AtomicBool = AtomicBool::new(false);

/// Terminal handler/wrapper, the piece of data that controls the terminal. Graphics and user input.
/// It used to be named Tui, for terminal user interface, in my previous
/// attempt. I kind of miss that special name, but it'd be confusing.
///
/// NCurses is unsafe and the terminal it controls is a global state. So any function that deals
/// ncruses requires a &mut Term argument so that Rust's borrow checker will prevent a concurrent
/// call on any function that deals with ncurses.
#[derive(Debug)]
pub struct Term {
	gem_width: u8,
	gem_height: u8,
	gap_width: u8, // "gap" is the empty space between tiles/gems.
	gap_height: u8,
	cursor_y: u8, // bad. to change
	cursor_x: u8,

	pub msg: &'static str
}

impl Term {
	pub fn new() -> Result<Term, &'static str> {

		// Test and set
		let f = NCURSES_FLAG.swap(true, Ordering::Acquire);
		if f == true {
			return Err("NCurses was already initialized by another Tui.");
		}

		let r = init_ncurses();
		if r.is_err() {return Err("NCurses could not be initialized correctly.");}



		return Ok(Term{
				gem_width: 4,
				gem_height: 2,
				gap_width: 2,
				gap_height: 1,
				cursor_y: 0,
				cursor_x: 0,
				msg: "Press 'w' to exit."
		});
	}

	/// Test whether there is enough space to draw a grid in the current terminal.
	/// Calling Term::draw() in too small a terminal is not an error and prints a message instead.
	pub fn can_draw(&self, g: &grid::Grid) -> bool {
		g.cols()*usize::from(self.gem_width) + (g.cols() - 1)*usize::from(self.gap_width) <= COLS() as usize
		&&
		g.lines()*usize::from(self.gem_height) + (g.lines() - 1)*usize::from(self.gap_height) + 2 <= LINES() as usize
	}

	pub fn draw(&mut self, grid: &grid::Grid) {
		erase();

		if ! self.can_draw(grid) {
			mvaddstr(LINES() / 2, 0, "The screen is too smol UwU");
			refresh();
			return;
		}

		let grid_height = grid.lines();
		let grid_width = grid.cols();

		// this function may change. A lot can be done to improve it. Se the comment

		wmove(stdscr(), 0, 0);
		for y in 0..grid_height {
			// draw self.gem_height colorful lines.
			for _ in 0..self.gem_height {
				for x in 0..grid_width {
					// draw the gem
					let c = ' ' as chtype | A_REVERSE() | color(grid.get(y, x));
					for _ in 0..self.gem_width {
						addch(c); // could write a char array instead...
					}
					// draw the gap
					for _ in 0..self.gap_width {
						addch(' ' as chtype); // could do a wmove() instead.
					}
				}
				// new line
				addch('\n' as chtype);
			}
			// the gap
			for _ in 0..self.gap_height {
				addch('\n' as chtype);
			}
		}
		// temporary
		addstr(self.msg);

		// draw the 'cursor', it highlights the current tile.
		let r = self.echo_cursor(grid, self.cursor_y, self.cursor_x, self.cursor_y, self.cursor_x);
		if let Err(e) = r {
			self.msg = e;
			addstr(self.msg);
		}

		refresh();
	}

	fn echo_cursor(&mut self,
		           grid: &grid::Grid,
		           old_y: u8,
				   old_x: u8,
				   new_y: u8,
				   new_x: u8) -> Result<(), &'static str> {
		let mut error = false;
		// Erase the previous cursor
		let y = old_y * (self.gem_height + self.gap_height);
		let x = old_x * (self.gem_width  + self.gap_width);
		error |= ERR == wmove(stdscr(), y.into(), x.into());
		error |= ERR == echochar (32 | A_REVERSE() | color(grid.get(old_y.into(),old_x.into())));
		// Echo the new one.
		let y = new_y * (self.gem_height + self.gap_height);
		let x = new_x * (self.gem_width  + self.gap_width);
		error |= ERR == wmove(stdscr(), y.into(), x.into());
		error |= ERR == echochar (ACS_DIAMOND() | A_REVERSE() | color(grid.get(new_y.into(), new_x.into())));

		if error {
			Err("Error when drawing the cursor. Probably due to position outside of bounds.")
		} else {
			Ok(())
		}
	}

	/// some refractor wouldn't hurt.
	pub fn process_input(&mut self, g: &grid::Grid) -> Option<i32> {
		// fetch user's input.
		// TODO if it's a weird char (eg å) then it won't fetch all the bytes.
		// look in the commented main() dead code below to see how to suck up all the bytes.
		let c = getch();
		// if it's a KEY, then with compute the new cursor position here.
		let new_cursor: Option<(u8, u8)> = match c {
			KEY_UP => {
				if self.cursor_y != 0 {
					Some((
						self.cursor_y - 1,
						self.cursor_x,
					))
				} else {
					Some((
						g.lines() as u8 - 1,
						self.cursor_x,
					))
				}
			}
			KEY_LEFT => {
				if self.cursor_x != 0 {
					Some((
						self.cursor_y,
						self.cursor_x - 1,
					))
				} else {
					Some((
						self.cursor_y,
						g.cols() as u8 - 1,
					))
				}
			}
			KEY_DOWN => {
				if usize::from(self.cursor_y) != g.lines() - 1 {
					Some((
						self.cursor_y + 1,
						self.cursor_x,
					))
				} else {
					Some ((
						0,
						self.cursor_x,
					))
				}
			}
			KEY_RIGHT => {
				if usize::from(self.cursor_x) != g.cols() - 1 {
					Some((
						self.cursor_y,
						self.cursor_x + 1,
					))
				} else {
					Some((
						self.cursor_y,
						0,
					))
				}
			}
			KEY_RESIZE => {
				erase();
				self.draw(g);
				None
			}
			_ => None
		};
		// phew, that's a big match

		if let Some((new_y, new_x)) = new_cursor { // The cursor shall move.
			let r = self.echo_cursor(
				g,
				self.cursor_y,
				self.cursor_x,
				new_y,
				new_x,
			);
			if let Err(s) = r {
				self.msg = s;
				return None;
			}
			// Then we update the cursor position in the data.
			self.cursor_y = new_y;
			self.cursor_x = new_x;
			return None;
		} else {
			// The user didn't move so we return the character he pressed.

			let byte: Result<u8, _> = c.try_into();
			if let Ok(byte) = byte {
				if byte.is_ascii() {
					return Some(c);
				} else {
					empty_stdin(self);
					return None;
				}
			} else { // This case is reached if the character fetched was e.g. an arrow key.
				return None;
			}
		}
	}

	/// Return the coordinates of the currently selected tile (or gem).
	/// It is (x, y) in the matrix convention.
	///
	/// Whether it returns u8 or usize still has to be decided.
	pub fn get_cursor(&self) -> (usize, usize) {
		return (self.cursor_y.into(), self.cursor_x.into());
	}

}

impl Drop for Term {
	fn drop(&mut self) {
		endwin();
		// release the init_flag lock
		NCURSES_FLAG.store(false, Ordering::Release);
	}
}

/// Suck up all the bytes in the buffer of the standard input throught NCurses's getch()
/// and return control once the buffer is empty. Panic if it deems the buffer was too full.
fn empty_stdin(_ncurses_guard: &mut Term) {
	// Non-blocking mode.
	if nodelay(stdscr(), true) == ERR {
		endwin();
		panic!("Couldn't set no-delay mode for stdin.");
	}
	for _ in 0..64 {
		if getch() == ERR {
			// Ok, the input buffer is empty, we can exit the function.
			// Back to blocking mode.
			if nodelay(stdscr(), false) == ERR {
				endwin();
				panic!("Couldn't set yes-delay mode for stdin.");
			}
			return;
		}
	}
	// If this part is reached, it means we should crash because the buffer was too full.
	// it is a conservative security feature. The user input is not trusted here.
	endwin();
	panic!("There are too many characters in the stdin buffer yo. Aborting. Wtf were u doing.");
}

// pub fn main() -> Result<i32, ()> {
// 	init_ncurses()?;
// 	addstr("Press 'q' to quit\n");
// 	loop {
// 		// Set blocking
// 		if nodelay(stdscr(), false) == ERR {
// 			endwin();
// 			return Err(());
// 		}
// 		let mut c = getch();
// 		if c == 'q' as i32 || c == ERR {
// 			break;
// 		}
// 		// Set non-blocking
// 		if nodelay(stdscr(), true) == ERR {
// 			endwin();
// 			return Err(());
// 		}
// //TODO add a compter to crash if the user provided 36 characters in one click. But it's buffered
// 		let mut vec = vec![0u8; 4];
// 		'tag: for i in 1..4 {
// 			let second = getch();
// 			if second == ERR {break 'tag;}
//
// 			let byte: u8 = second.try_into().unwrap_or_else(|_| {
// 						endwin(); panic!("that char was bigger than a c char")});
//
// 			// unsafe {
// 			// 	let pointer = (&mut c as *mut i32).cast::<u8>();
// 			// 	pointer.add(i).write(byte);
// 			// }
// 			vec[i] = byte;
// 		}
//
// 		if 0 < c && c < 255 {
// 			vec[0] = (c & 0x000000FF) as u8;
// 		}
// 		let s = String::from_utf8(vec).unwrap_or_else(|e|{
// 			let mut s = String::new();
// 			write!(&mut s, "{}", e).unwrap();
// 			return s;
// 		});
// 		let ch = s.chars().next();
// 		let mut string = String::new();
// 		writeln!(&mut string, "{} (unicode: {:?})", c, ch).unwrap();
// 		addstr(&string[..]);
// 	}
//
//
// 	endwin();
// 	return Ok('F' as i32);
// }


// fn sub_window() -> Result<WINDOW, ()> {
// 	let w = subwin(stdscr(), 10, 10, 10, 10);
// 	if w.is_null() {return Err(());}
//
// 	mvwaddch(w, 0,0, 'B' as chtype | A_BOLD() | A_UNDERLINE() | COLOR_PAIR(3) | A_REVERSE());
// 	touchwin(stdscr());
// 	wrefresh(w);
// 	Ok(w)
// }

fn init_ncurses() -> Result<(), ()> {
	let r = initscr(); // Initializes stuff and put the terminal in that screen mode.
	if r.is_null() {
		eprintln!("Failed to initialize the terminal interface. Aborting.");
		return Err(());
	}
	let mut r = false;
	r |= ERR == cbreak(); // No char buffering. NCurses sucks typed chars immediately.
	r |= ERR == noecho(); // Do not print the typed character on the terminal.
	r |= ERR == keypad(stdscr(), true); // Allow capture of the arrow keys and others.
	r |= ERR == nonl(); // Turn '\n' into '\r' when printed. "no newline". Not useful but anyway.
	// r |= clearok(stdscr(), false); ???
	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE); // invisible cursor
	if r {
		endwin();
		eprintln!("sh!t happened: ncurses was not correctly initialized.");
		return Err(());
	}
	if !has_colors() {
		endwin();
		eprintln!("No color! You can't play with no color, exiting.");
		return Err(());
	}
	start_color();
	wmove(stdscr(), 0, 0);

	init_pair(1, COLOR_WHITE, COLOR_BLACK);
	init_pair(2, COLOR_RED, COLOR_BLACK);
	init_pair(3, COLOR_GREEN, COLOR_BLACK);
	init_pair(4, COLOR_YELLOW, COLOR_BLACK);
	init_pair(5, COLOR_BLUE, COLOR_BLACK);
	init_pair(6, COLOR_MAGENTA, COLOR_BLACK);
	init_pair(7, COLOR_CYAN, COLOR_BLACK);
	init_pair(8, COLOR_BLACK, COLOR_WHITE);

	erase(); // ensure the screen starts blank.

	Ok(())
}

fn color(gem: grid::Gem) -> chtype {
	match gem {
		grid::Gem::Green => COLOR_PAIR(3),
		grid::Gem::Red => COLOR_PAIR(2),
		grid::Gem::Yellow => COLOR_PAIR(4),
		grid::Gem::Blue => COLOR_PAIR(5),
		grid::Gem::White => COLOR_PAIR(1),
		grid::Gem::Pink => COLOR_PAIR(6),
		grid::Gem::Cyan => COLOR_PAIR(7),
	}
}
