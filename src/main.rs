use configparser::ini::Ini;
use git2::Repository;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;

fn main() {
	let path: PathBuf = dirs::home_dir()
		.expect("Failed to access home directory")
		.join(".cru");
	// Create a new repository if it doesn't exist
	if !path.is_dir() {
		// Path doesn't exist
		firstLaunch(&path);
	} else {
		// Path exists
	}
	let repo = match Repository::open(&path) {
		Ok(repo) => repo,
		Err(e) => panic!("Failed to open a git repository: {}", e),
	};
	let mut config = Ini::new();
	let confMap = config.load(path.join("conf")).expect("Failed to load config file");
}

fn firstLaunch(path: &PathBuf) {
	println!(
		"Working directory doesn't exist yet. Do you wish to:
[1] create a new repository for records under \"{}\"
[2] clone an existing repository
[3] abort
Please choose your option (1-3):",
		path.display()
	);
	let mut ans = String::new();
	io::stdin()
		.read_line(&mut ans)
		.expect("Failed to read line");
	match ans.trim() {
		"1" => {
			fs::create_dir_all(path.clone())
				.expect("Failed to create working directory. Please check the permissions.");
			let repo = match Repository::init(path) {
				Ok(repo) => repo,
				Err(e) => panic!("Failed to init a git repository: {}", e),
			};
			createConfig(&path);
		}
		"2" => {
			println!("Provide an address of your repository:");
			let mut address = String::new();
			io::stdin()
				.read_line(&mut address)
				.expect("Failed to read line");
			fs::create_dir_all(path.clone())
				.expect("Failed to create working directory. Please check the permissions.");
			match Repository::clone(&address, path) {
				Ok(repo) => repo,
				Err(e) => panic!("failed to clone: {}", e),
			};
		}
		"3" => {
			std::process::exit(0);
		}
		_ => {
			println!("Invalid option.");
			firstLaunch(path);
		}
	}
}

fn createConfig(path: &PathBuf) {
	let mut conffile = fs::File::create(path.join("conf")).expect("Failed to create config file");
	write!(&mut conffile, "[GIT SETTINGS]\nremote = no").expect("Failed to write to config file");
}
