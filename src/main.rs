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

	let mut conf = game::Config::default();
	conf.height = 8;
	conf.width = 6;
	game::main(&conf).unwrap();

}
