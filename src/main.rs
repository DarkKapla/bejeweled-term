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

	// Set a custom panic that first attempts to uninitialize NCurses before printing any message.
	let old_hook = std::panic::take_hook();
	std::panic::set_hook(Box::new(move |panic_info| {
		term::free_ncurses();
		old_hook(panic_info);
		std::process::exit(1); // for now, panicking terminates the process.
	}));

	let mut conf = game::Config::default();
	conf.height = 8;
	conf.width = 6;
	game::main(&conf).unwrap();

}
