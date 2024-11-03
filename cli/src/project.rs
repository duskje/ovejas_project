use std::path::PathBuf;
use std::env;

fn is_project_root(current_dir: &PathBuf) -> bool {
    for file in current_dir.read_dir().expect("Could not read directory") {
        let file_path = file.expect("Could not read file").path();
        let file_name = file_path.file_name().expect("Could not read file");
        let file_name = file_name.to_str().expect("Could not read file");

        if file_name == "pyproject.toml" {
            return true;
        }
    }

    false
}

pub fn find_project_root() -> Option<String> {
    let home_dir = env::home_dir()?;
    let mut current_dir = env::current_dir().ok()?;

    if is_project_root(&current_dir) {
        return Some(String::from(current_dir.to_str()?));
    }

    while home_dir != current_dir {
        current_dir = current_dir.parent()?.to_path_buf();

        if is_project_root(&current_dir) {
            return Some(String::from(current_dir.to_str()?));
        }
    }

    None
}
