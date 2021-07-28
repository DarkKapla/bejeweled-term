//!
//!
//!

// game logic and rules
mod game;
// basic operation on the jewel grid
mod grid;
// control the terminal user interface
mod term;

fn main() {

	// let c = term::main().unwrap();
	// let ch: char = unsafe { std::ptr::read(&c as *const i32 as *const char)};
	//
	// println!("{} ({})", c, ch);//((c & 0xFF) as u8) as char);
	let mut conf = game::Config::default();
	conf.height = 8;
	conf.width = 6;
	game::main(&conf).unwrap();

	// let mut term = term::Term::new().unwrap();
	// let mut g = grid::Grid::new_rand(5, 5);
	// //ncurses::attr_on(ncurses::A_BOLD());
	// term.draw(&g);
	// let mut o = Option::<i32>::None;
	// while o == None {
	// 	o = term.process_input(&g);
	// }
	// drop(term);
	// eprintln!("{:?}", o);

	// let mut t = view::Term::new().unwrap();
	// t.draw(&g);
	// ncurses::getch();
// 	let mut g = grid::Grid::new_rand(5, 15);
//
// 	loop {
// 		for row in g.0.rows() {
// 			for gem in row {
// 				print!("{} ", gem_to_char(*gem));
// 			}
// 			println!("\x1b[00m");
// 		}
//
// 		let (v, m) = g.get_all_matches();
//
// 		println!("Matches: {:?}\n", &m);
// 		if v.is_empty() {break;}
//
// 		g.destroy_gems(&v);
//
// 	}
//
// }

}
