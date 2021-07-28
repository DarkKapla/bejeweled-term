/*
 * Handles the game logic.
 */


use std::default::Default;

static mut REMOVE_ME: [u8; 128] = [0; 128];

#[derive(Clone, Copy, Debug)]
pub struct Config {
	pub width: u8,
	pub height: u8,

}

impl Default for Config {
	fn default() -> Self {
		Config {
			width: 7,
			height: 7
		}
	}
}

///
pub fn main(conf: &Config) -> Result<(), &str> {

	let mut grid = crate::grid::Grid::new_rand(conf.height, conf.width);
	let mut term = crate::term::Term::new()?;

	// TODO refractor this. It's a bal of mud.

	// clean the grid of any match
	for i in 0..64 {
		if i == 63 {
			return Err("Couldn't clean the grid of its matches.");
		}
		let (v, _) = grid.get_all_matches();
		if v.is_empty() {
			break;
		}
		grid.destroy_gems(&v);
	}

	loop {

		term.draw(&grid);

		// Call the process input routine until it returns something interesting for us.
		let char = loop {
			if let Some(c) = term.process_input(&grid) {
				break c;
			}
		};

		// Convert the i32 into a char.
		let char = {
			if char & (!0x7F_i32) != 0 {
				return Err("Non ascii characters are not supported.");
			}
			let u = char & 0x7F_i32;
			unsafe {*(&u as *const i32).cast::<char>()}
		};

		// exit
		if char == 'w' {
			break;
		}
		// move two gems
		else if ['z', 'q', 's', 'd'].contains(&char) {
			let (x, y) = term.get_cursor();
			let (x2, y2) = match char {
				'z' if x != 0 => (x - 1, y),
				'q' if y != 0 => (x, y - 1),
				's' if x != grid.lines() - 1 => (x + 1, y),
				'd' if y != grid.cols() - 1  => (x, y + 1),
				_ => continue,
			};

			grid.permute((x, y), (x2, y2));
			if grid.check_matches((x, y), (x2, y2)) {

				// There, we can finally play.
				let (gems, mut matches) = grid.get_all_matches();
				// update the grid
				grid.destroy_gems(&gems);
				// there can be new matches formed.
				for lvl in 0.. {
					let (g, mut m) = grid.get_all_matches();
					if g.is_empty() {
						break;
					}
					term.draw(&grid);
					grid.destroy_gems(&g);
					matches.append(&mut m);
					std::thread::sleep(std::time::Duration::from_secs(1));

					// print the level. Plz rework on that later.
					unsafe {
						let s = format!("{}...", lvl);
						let bytes = s.as_bytes();
						let l = usize::min(bytes.len(), REMOVE_ME.len());
						let dest = &mut REMOVE_ME[..l];
						dest.copy_from_slice(bytes);
						term.msg = std::str::from_utf8(dest).unwrap_or("Fatal error #D6tedD54EGD");
					}
				}

				// and compute the score
				let score: f32 = matches.into_iter()
				                        .map(|(_gem, len)| {f32::from(len)*f32::from(len) / 9.})
										.sum();
				// print the score. Plz rework on that later.
				unsafe {
					let s = format!("Score of {}, great!", score);
					let bytes = s.as_bytes();
					let l = usize::min(bytes.len(), REMOVE_ME.len());
					let dest = &mut REMOVE_ME[..l];
					dest.copy_from_slice(bytes);
					term.msg = std::str::from_utf8(dest).unwrap_or("Fatal error #D6tedD54EGD");
				}
			} else {
				grid.permute((x, y), (x2, y2));
				term.msg = "No match!";
			}

		}


	}

	drop(term); // always drop term before making use of stdout or stderr.
	println!("End.");
	return Ok(());
}
