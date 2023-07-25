use configparser::ini::Ini;
use git2::Repository;
use std::path::PathBuf;

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

mod interface {
	use ansi_term::Colour;
	use chrono::Utc;
	use clap::{arg, Command};
	use configparser::ini::Ini;
	use edit;
	use git2::Repository;
	use std::fs;
	use std::io;
	use std::io::Read;
	use std::path::PathBuf;
	pub fn first_launch(path: &PathBuf) {
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
				git_integration::create_new_repo(&path);
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

	pub fn parse_cli() -> Command {
		clap::command!()
			.about("cru - crude record utility")
			.arg_required_else_help(true)
			.subcommand(
				Command::new("new")
					.alias("n")
					.about("Create new note")
					.arg(arg!(<NAME> "Name of the note").required(false)),
			)
			.subcommand(Command::new("ls").alias("l").about("List all notes"))
			.subcommand(
				Command::new("edit")
					.alias("e")
					.about("Edit existing note")
					.arg(arg!(<NAME> "Name of the note").required(true)),
			)
			.subcommand(
				Command::new("show")
					.alias("s")
					.about("Show existing note")
					.arg(arg!(<NAME> "Name of the note").required(true)),
			)
	}

	pub fn new_note(argument: &clap::ArgMatches, repo: &Repository, path: &PathBuf) {
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
		edit::edit_file(path.join("records.d").join(&name))
			.expect("Failed to open default text editor");
		git_integration::commit(&repo, &name);
	}

	pub fn ls_notes(repo: &Repository, path: &PathBuf) {
		let mut record_list = Ini::new();
		record_list
			.load(path.join("records"))
			.expect("Failed to open records file");
		let sections = record_list.sections();
		for record in sections.iter() {
			println!(
				"{}   {}",
				Colour::Blue.bold().paint(
					record_list
						.get(record, "modified")
						.expect("Corrupted records file")
				),
				Colour::Red.bold().paint(record),
			);
		}
	}

	pub fn edit_note(argument: &clap::ArgMatches, repo: &Repository, path: &PathBuf) {
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
		edit::edit_file(path.join("records.d").join(&name))
			.expect("Failed to open default text editor");
		record_list.set(&name, "modified", Some(now.to_string()));
		record_list
			.write(path.join(&name))
			.expect("Failed to write to records file");
		git_integration::commit(&repo, &name);
	}

	pub fn show_note(argument: &clap::ArgMatches, repo: &Repository, path: &PathBuf) {
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
		// Read data and metadata
		let record =
			fs::File::open(path.join("records.d").join(&name)).expect("Failed to open record");
		let mut record_buf = io::BufReader::new(record);
		let mut record_str = String::new();
		record_buf.read_to_string(&mut record_str);
		print!(
			"{}\n{}\n\n{}",
			Colour::Red.bold().paint(&name),
			Colour::Blue.bold().paint(
				record_list
					.get(&name, "modified")
					.expect("Corrupted records file")
			),
			record_str,
		);
	}

	mod git_integration {
		use git2::Repository;
		use std::fs;
		use std::io::Write;
		use std::path::PathBuf;
		pub(in crate::interface) fn commit(repo: &Repository, message: &str) {
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

		pub(in crate::interface) fn create_new_repo(path: &PathBuf) {
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
			let mut conffile =
				fs::File::create(path.join("conf")).expect("Failed to create config file");
			write!(&mut conffile, "[GIT SETTINGS]\nremote = no")
				.expect("Failed to write to config file");
			fs::File::create(path.join("records")).expect("Failed to create records file");
		}
	}
}
