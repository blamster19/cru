use configparser::ini::Ini;
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

pub(in crate::interface) fn add_remote(repo: &Repository, config_path: &PathBuf, url: &str) {
	// add remote to config
	let mut config = Ini::new();
	config
		.load(config_path.join("conf"))
		.expect("Failed to open config file");
	config.set("GIT SETTINGS", "remote", Some(url.to_string()));
	config
		.write(config_path.join("conf"))
		.expect("Failed to write to config file");
	// add remote to git
	repo.remote("origin", &url)
		.expect("Failed to add remote repository");
	commit(&repo, "add remote repository");
}

fn create_config(path: &PathBuf) {
	let mut conffile = fs::File::create(path.join("conf")).expect("Failed to create config file");
	fs::File::create(path.join("records")).expect("Failed to create records file");
}
