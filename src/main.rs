use chrono::Utc;
use clap::{arg, Command};
use configparser::ini::Ini;
use edit;
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
		first_launch(&path);
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
	let matches = parse_cli().get_matches();
	match matches.subcommand() {
		Some(("new", sub_matches)) => new_note(sub_matches, &repo, &path),
		Some(("ls", sub_matches)) => ls_notes(&repo, &path),
		Some(("edit", sub_matches)) => edit_note(sub_matches, &repo, &path),
		_ => unreachable!(),
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
			create_new_repo(&path);
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

fn commit(repo: &Repository, message: &str) {
	let mut index = repo.index().expect("Failed to get Index file");
	index
		.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
		.expect("Failed to commit");
	index.write().expect("Failed to commit");
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
			)
			.expect("Failed to commit");
		}
		Err(_e) => {
			repo.commit(Some("HEAD"), &signature, &signature, message, &tree, &[])
				.expect("Failed to commit");
		}
	};
}

fn create_new_repo(path: &PathBuf) {
	fs::create_dir_all(path.clone())
		.expect("Failed to create working directory. Please check the permissions.");
	let repo = match Repository::init(path) {
		Ok(repo) => repo,
		Err(e) => panic!("Failed to init a git repository: {}", e),
	};
	create_config(&path);
	std::fs::create_dir_all(path.join("records.d")).expect("Failed to create directory");
	commit(&repo, "Initial commit");
}

fn create_config(path: &PathBuf) {
	let mut conffile = fs::File::create(path.join("conf")).expect("Failed to create config file");
	write!(&mut conffile, "[GIT SETTINGS]\nremote = no").expect("Failed to write to config file");
	fs::File::create(path.join("records")).expect("Failed to create records file");
}

fn parse_cli() -> Command {
	Command::new("cru")
		.about("crude record utility")
		.arg_required_else_help(true)
		.subcommand(
			Command::new("new")
				.about("Create new note")
				.arg(arg!(<NAME> "Name of the note").required(false)),
		)
		.subcommand(Command::new("ls").about("List all notes"))
		.subcommand(
			Command::new("edit")
				.about("Edit existing note")
				.arg(arg!(<NAME> "Name of the note").required(true)),
		)
}

fn new_note(argument: &clap::ArgMatches, repo: &Repository, path: &PathBuf) {
	// Prepare metadata to write
	let now = Utc::now().to_rfc3339();
	let now = now.as_str();
	let name: String = match argument.get_one::<String>("NAME") {
		Some(name) => name,
		None => now,
	}
	.to_string();
	// Register in records file
	let mut record_list = Ini::new();
	record_list
		.load(path.join("records"))
		.expect("Failed to open records file");
	// Check if name isn't taken
	if record_list.sections().contains(&name) {
		println!("Note with such name already exists!");
		std::process::exit(0);
	}
	// Add record
	record_list.set(&name, "modified", Some(now.to_string()));
	record_list
		.write(path.join("records"))
		.expect("Failed to write to records file");
	let mut record =
		fs::File::create(path.join("records.d").join(&name)).expect("Couldn't create a record");
	edit::edit_file(path.join("records.d").join(&name))
		.expect("Failed to open default text editor");
	commit(&repo, &name);
}

fn ls_notes(repo: &Repository, path: &PathBuf) {
	let mut record_list = Ini::new();
	record_list
		.load(path.join("records"))
		.expect("Failed to open records file");
	let sections = record_list.sections();
	for record in sections.iter() {
		println!(
			"{}   {}",
			record_list
				.get(record, "modified")
				.expect("Corrupted records file"),
			record,
		);
	}
}

fn edit_note(argument: &clap::ArgMatches, repo: &Repository, path: &PathBuf) {
	let name: String = argument
		.get_one::<String>("NAME")
		.expect("Failed to read name")
		.to_string();
	let mut record_list = Ini::new();
	record_list
		.load(path.join("records"))
		.expect("Failed to open records file");
	// Check if record exists
	if !record_list.sections().contains(&name) {
		println!("Note with such name doesn't exist!");
		std::process::exit(0);
	}
	let now = Utc::now().to_rfc3339();
	let now = now.as_str();
	let mut record =
		fs::File::open(path.join("records.d").join(&name)).expect("Couldn't open a record");
	edit::edit_file(path.join("records.d").join(&name))
		.expect("Failed to open default text editor");
	record_list.set(&name, "modified", Some(now.to_string()));
	record_list.write(path.join(&name));
	commit(&repo, &name);
}
