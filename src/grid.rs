/*
 * The grid. Raw data here, but also a bit of logic.
 */

// ndarray doc
// https://docs.rs/ndarray/0.15.1/ndarray/
use ndarray::Array2;
use rand::Rng;

use std::convert::TryInto;
use std::default::Default;

// The game is called match the three, after all.
const NUMBER_TO_MATCH: u8 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gem {
	Green,
	Red,
	Yellow,
	Blue,
	White,
	Pink,
	Cyan
}

impl Default for Gem {
	fn default() -> Gem {Gem::Green}
}

impl Gem {
	fn from_u8(x: u8) -> Gem {
		// x modulo 7 and then the matching.
		match x % 7 {
			0 => Gem::Green,
			1 => Gem::Red,
			2 => Gem::Yellow,
			3 => Gem::Blue,
			4 => Gem::White,
			5 => Gem::Pink,
			6 => Gem::Cyan,
			_ => unreachable!()
		}
	}
}

/*
╔═══╤═══╤═══╗
║   │   │   ║
╟───┼───┼───╢
║   │   │   ║
╟───┼───┼───╢
║   │   │   ║
╚═══╧═══╧═══╝
*/

// A single-nameless-element struct.
pub struct Grid (pub Array2<Gem>);

impl std::fmt::Debug for Grid {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Grid of gems:\n{:?}", self.0)
	}
}

impl Grid {

	pub fn new_from(lines: u8, cols: u8, gem: Gem) -> Grid {
		if lines < 2 || cols < 2 {
			panic!("Grid's width and height shall not be smaller than 2.");
		}

		Grid(Array2::<Gem>::from_elem((lines.into(), cols.into()), gem))
	}

	pub fn new_rand(lines: u8, cols: u8) -> Grid {

		let mut g = Grid::new_from(lines, cols, Gem::Green);

		let mut rng = rand::thread_rng();
		for gem in g.0.iter_mut() {
			*gem = Gem::from_u8(rng.gen());
		}
		return g;
	}

	// pub fn size(&self) -> (usize, usize) {
	// 	let s = self.0.shape();
	// 	(s[0], s[1])
	// } // remove if it's never needed.
	#[inline]
	pub fn lines(&self) -> usize {
		self.0.shape()[0]
	}
	#[inline]
	pub fn cols(&self) -> usize {
		self.0.shape()[1]
	}
	#[inline]
	pub fn get(&self, x:usize, y:usize) -> Gem {
		self.0[[x, y]]
	}

	pub fn permute(&mut self, (x1, y1): (usize, usize), (x2, y2): (usize, usize)) {
		let tmp = self.0[[x1, y1]];
		self.0[[x1, y1]] = std::mem::replace(&mut self.0[[x2, y2]], tmp);
	}
	/** Returns (u0, u1, u2, u3) where

                     u1
	                 |
	       u0 <--- root ---> u2
                     |
					u3

	 * There are indexes of the furthest gems that match the colour of point.
	 * If u0 equals root_x, it means that the gem on top of root is different, or that
	 * root is a the top of the grid already.

	 * Another name could be (y_start, x_start, y_end, x_end)
	 * This function is useful to check if a player's move triggers matches or not.
	 */
	fn match_border_from_point(&self, root_x: usize, root_y: usize) -> (u8,u8,u8,u8) {
		let gem = self.0[[root_x, root_y]];

		let o0 = (0..=root_y).rev()    .skip_while(|&y| self.0[[root_x, y]] == gem).next();
		let o1 = (0..=root_x).rev()    .skip_while(|&x| self.0[[x, root_y]] == gem).next();
		let o2 = (root_y..self.cols()) .skip_while(|&y| self.0[[root_x, y]] == gem).next();
		let o3 = (root_x..self.lines()).skip_while(|&x| self.0[[x, root_y]] == gem).next();

		return (
			o0.and_then(|y| Some(y + 1)).unwrap_or(0)             .try_into().unwrap(),
			o1.and_then(|x| Some(x + 1)).unwrap_or(0)             .try_into().unwrap(),
			o2.and_then(|y| Some(y - 1)).unwrap_or(self.cols()-1) .try_into().unwrap(),
			o3.and_then(|x| Some(x - 1)).unwrap_or(self.lines()-1).try_into().unwrap(),
		);
	}

	/// Tests whether the two points are in any match or not.
	/// After a move from the player, call this function to know if it a valid move.
	pub fn check_matches(&self, (x1, y1): (usize, usize), (x2, y2): (usize, usize)) -> bool {
		let (y1_start, x1_start, y1_end, x1_end) = self.match_border_from_point(x1, y1);
		let (y2_start, x2_start, y2_end, x2_end) = self.match_border_from_point(x2, y2);

		x1_end - x1_start + 1 >= NUMBER_TO_MATCH ||
		y1_end - y1_start + 1 >= NUMBER_TO_MATCH ||
		x2_end - x2_start + 1 >= NUMBER_TO_MATCH ||
		y2_end - y2_start + 1 >= NUMBER_TO_MATCH
	}

	/// From one point, you may get a cluster of matches. Connex, and in a shape of either a cross
	/// or a bar (a cross being two bars). The center of the cross has to be the root point.
	///
	///        x x o x x
	///        o o(o)o x
	///        x x o x x
	///
	/// This function inserts in the provided BTreeSet all the new points in the matches.
	// fn get_match_from_point(&self, x: usize, y: usize,
	// 				collec: &mut BTreeSet<(u8, u8)>, matches: &mut Vec<(Gem, u8)>) {
	//
	// 	let (y_start, x_start, y_end, x_end) = self.match_border_from_point(x, y);
	//
	// 	if x_end - x_start + 1 >= NUMBER_TO_MATCH {
	// 		// Check whether ALL the points in the new match already are in the BTreeSet.
	// 		// However, they could be in but in other orthogonal matches...
	// 		for i in x_start..=x_end {
	// 			collec.insert((i as u8, y as u8));
	// 		}
	// 	}
	//
	// 	if y_end - y_start + 1 >= NUMBER_TO_MATCH {
	// 		for j in y_start..=y_end {
	// 			collec.insert((x as u8, j as u8));
	// 		}
	// 	}
	// }

	/// Probes the entire grid and returns two vectors:
	/// - the first one contains all points (x, y) that are in a matches are should be destroyed.
	///   (intended to feed destroy_gems())
	/// - the second one contains all matches (gem, length) horizontal and vertical.
	/// No duplicate. The vectors will be empty if no match be.
	pub fn get_all_matches(&self) -> (Vec<(u8, u8)>, Vec<(Gem, u8)>) {

		// Initialise the vector of matched points with capacity 3, the least possible.
		let mut points: Vec<(u8, u8)> = Vec::with_capacity(3);
		// Initialise the vector of matches with capacity 1, as most rounds will see only 1 match.
		let mut matches: Vec<(Gem, u8)> = Vec::with_capacity(1);

		let mut accumulator = Vec::with_capacity(3);
		let mut count = 1u8; // There cannot be over 255 matched gems in a 255×255 board.
		let mut old_gem = Default::default();
		// This variable is not given to the closure. It'll be sent to it as an argument.
		// It allows to prevent a match crossing over two lines or two columns, but not only.
		let mut match_is_continuous: bool;

		// This function performs the core of the logic: counting the amount of previous
		// similar gems and if there are enough of them, insert a new match.
		//
		// It is a closure because it shall be called twice: once when iterating horizontally,
		// (y moves faster compared to x) and once when iterating vertically (x moves faster).
		// so putting the logic in a closure avoids code duplication.
		let mut closure = |x: u8,
 			               y: u8,
						   gem: Gem,
						   match_is_continuous: bool,| {

			if match_is_continuous && old_gem == gem {

				accumulator.push((x, y));
				count += 1;

			} else {

				if count >= NUMBER_TO_MATCH {
					// It's a match!
					matches.push((old_gem, count));
					points.append(&mut accumulator);
				}

				accumulator.clear();
				accumulator.push((x, y));
				count = 1;
				old_gem = gem;
			}
		};

		// Horizontal
		for (x, r) in self.0.rows().into_iter().enumerate() {
			// We enter in a new line, so there can't be a match with the previous tile.
			match_is_continuous = false;
			for (y, gem) in r.into_iter().enumerate() {

				closure(x as u8, y as u8, *gem, match_is_continuous);
				match_is_continuous = true; // Next gems are (maybe) continuous.

			}
		}
		// We need a last check in case the last gems were in a match.
		// Call the clusore on an unmatchable dummy value.
		// In this call the closure will also reset count and accumulator, which is crucial.
		match_is_continuous = false;
		closure(0, 0, Gem::Red, match_is_continuous);


		// Vertical
		for (y, c) in self.0.columns().into_iter().enumerate() {
			// We enter in a new column, so there shan't be a match with the previous tile.
			match_is_continuous = false;
			for (x, gem) in c.into_iter().enumerate() {

				closure(x as u8, y as u8, *gem, match_is_continuous);
				match_is_continuous = true;

			}
		}
		match_is_continuous = false;
		closure(0, 0, Gem::Red, match_is_continuous);

		// Remove duplicate in points
		points.sort();
		points.dedup();

		return (points, matches);
	}

	pub fn destroy_gems(&mut self, to_destroy: &[(u8, u8)]) {

		// Not optimised.

		let mut rng = rand::thread_rng();

		for &(x, y) in to_destroy {
			// Destroy the gem by moving all that's up down on tile.
			for up_x in (0..x).rev() { // from x-1 to 0.
				self.0[[up_x as usize + 1, y.into()]] = self.0[[up_x.into(), y.into()]];
			}
			// generate the new top gem
			self.0[[0, y.into()]] = Gem::from_u8(rng.gen());
		}
	}


}

// Because TDD

#[cfg(test)]
mod test {
	use super::*;
	use super::Gem::*;
	use std::iter::once;

	#[test]
	fn gems() {
		let mut gems = [Gem::Green, Gem::Red, Gem::Yellow, Gem::Blue].iter().cycle();
		for i in 0..u8::MAX {
			//eprintln!("{} ~> {:?}", &i, &Gem::from_u8(i));
			assert_eq!(Some(Gem::from_u8(i)).as_ref(), gems.next());
		}
	}

	#[test]
	fn basic_grid() {
		let g = Grid::new_from(5, 6, Gem::Green);
		assert_eq!(g.size(), (5, 6));
		assert_eq!(g.lines(), 5);
		assert_eq!(g.cols(), 6);
		let g = Grid::new_from(13, 2, Gem::Green);
		assert_eq!(g.size(), (13, 2));
		assert_eq!(g.lines(), 13);
		assert_eq!(g.cols(), 2);

		for size in 3..10 {
			let mut g = Grid::new_from(size, size, Gem::Green);


			g.0[[(size/2).into(), (size/2).into()]] = Gem::Red;
			let t = g.match_border_from_point((size/2) as usize, (size/2) as usize);
			assert_eq!(t, (size/2, size/2,size/2, size/2));

			let g = Grid::new_from(size, size, Gem::Blue);
			let t = g.match_border_from_point(0, 0);

		}

		for lines in (3..9).chain(once(30)) {
			for cols in (3..9).chain(once(30)) {

				let mut g = Grid::new_from(lines, cols, Gem::Yellow);

				for i in (0..(lines-1)).map(|a| a as usize) {
					for j in (0..(cols-1)).map(|a| a as usize) {
						let t = g.match_border_from_point(i, j);
						assert_eq!(t, (0, 0, cols-1, lines-1));
					}
				}
			}
		}
	}

	#[test]
	fn matches() {

		let g = Grid(ndarray::array![
			[Red,    Red,    Red,    Blue,   Blue,  Blue   ],
			[Green,  Green,  Yellow, Yellow, Red,    Blue   ],
			[Yellow, Yellow, Yellow, Yellow, Yellow, Blue   ],
			[Red,    Yellow, Yellow, Yellow, Yellow, Yellow ]
		]);

		let (vec, matches) = g.get_all_matches();
		println!("{:?}\n{:?}", vec, matches);
		for tuple in &[(Red, 3), (Blue, 3), (Yellow, 5),
		               (Yellow, 5), (Yellow, 3), (Yellow, 3), (Blue, 3)] {
						   assert!(matches.contains(tuple));
		}

		let g = Grid(ndarray::array![
			[Red,    Green, Green, Blue,   Blue],
			[Red,    Red,   Green, Yellow, Blue],
			[Yellow, Green, Blue,  Blue,   Yellow],
		]);
		let (vec, matches) = g.get_all_matches();
		assert!(vec.is_empty());
		assert!(matches.is_empty());

	}
}
