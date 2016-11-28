use find_folder;
use std::path::PathBuf;

pub fn path() -> PathBuf {
	find_folder::Search::KidsThenParents(0,0).for_folder("assets").unwrap()
}

pub fn tiles() -> PathBuf {
	path().join("images/tiles")
}