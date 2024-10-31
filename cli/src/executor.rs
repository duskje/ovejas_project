use std::process::Command;

pub fn state_from_program() {
    let output = Command::new("python")
        .arg("../main.py");
}
