use find_folder;
use std::path::PathBuf;

pub fn path() -> PathBuf {
	find_folder::Search::KidsThenParents(0,0).for_folder("assets").unwrap()
}

pub fn images()-> PathBuf {
	path().join("images")
}

pub fn image(name: &str) -> PathBuf {
	images().join(name)
}

pub fn tiles() -> PathBuf {
	images().join("tiles")
}