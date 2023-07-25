use configparser::ini::Ini;
use git2::Repository;
use std::path::PathBuf;
mod interface;

fn main() {
	let path: PathBuf = dirs::home_dir()
		.expect("Failed to access home directory")
		.join(".cru");
	// Create a new repository if it doesn't exist
	if !path.is_dir() {
		// Path doesn't exist
		interface::first_launch(&path);
	} else {
		// Path exists
	}
	let repo = match Repository::open(&path) {
		Ok(repo) => repo,
		Err(e) => panic!("Failed to open a git repository: {}", e),
	};
	let mut config = Ini::new();
	let _conf_map = config
		.load(path.join("conf"))
		.expect("Failed to load config file");
	// Parse arguments
	let matches = interface::parse_cli().get_matches();
	match matches.subcommand() {
		Some(("new", sub_matches)) => interface::new_note(sub_matches, &repo, &path),
		Some(("ls", sub_matches)) => interface::ls_notes(&repo, &path),
		Some(("edit", sub_matches)) => interface::edit_note(sub_matches, &repo, &path),
		Some(("show", sub_matches)) => interface::show_note(sub_matches, &repo, &path),
		_ => unreachable!(),
	}
}
