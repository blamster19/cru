use configparser::ini::Ini;
use git2::Repository;
use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};

fn main() {
	let path: PathBuf = dirs::home_dir()
		.expect("Failed to access home directory")
		.join(".cru");
	// Create a new repository if it doesn't exist
	if !path.is_dir() {
		// Path doesn't exist
		first_launch(&path);
	} else {
		// Path exists
	}
	let repo = match Repository::open(&path) {
		Ok(repo) => repo,
		Err(e) => panic!("Failed to open a git repository: {}", e),
	};
	let mut config = Ini::new();
	let conf_map = config
		.load(path.join("conf"))
		.expect("Failed to load config file");
	// Parse arguments
	let matches = parse_cli().get_matches();
	match matches.subcommand() {
		Some(("new", sub_matches)) => {
			println!("Adding new note");
		}
		_ => unreachable!()
	}
}

fn first_launch(path: &PathBuf) {
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
			create_config(&path);
			commit(&repo, "Initial commit");
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
			first_launch(path);
		}
	}
}

fn create_config(path: &PathBuf) {
	let mut conffile = fs::File::create(path.join("conf")).expect("Failed to create config file");
	write!(&mut conffile, "[GIT SETTINGS]\nremote = no").expect("Failed to write to config file");
}

fn commit(repo: &Repository, message: &str) {
	let mut index = repo.index().expect("Failed to get Index file");
	index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None);
	index.write();
	let signature =
		git2::Signature::now("cru", "cru@xyz.net").expect("Failed to create a signature");
	let oid = index.write_tree().expect("Couldn't write tree");
	let tree = repo.find_tree(oid).expect("Couldn't find tree");
	match repo.head() {
		Ok(head) => {
			let parent = head
				.resolve()
				.unwrap()
				.peel_to_commit()
				.expect("Couldn't find last commit");
			repo.commit(
				Some("HEAD"),
				&signature,
				&signature,
				message,
				&tree,
				&[&parent],
			);
		}
		Err(e) => {
			repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[]);
		}
	};
}

fn parse_cli() -> Command {
	Command::new("cru")
		.about("crude record utility")
		.arg_required_else_help(true)
		.subcommand(
			Command::new("new")
				.about("Create new note")
		)
}
