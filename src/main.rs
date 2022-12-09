use std::io;

use fat::FAT;

mod fat;
mod cli;

struct Application {
    current_path: String,
    file_system: FAT,
}

impl Application {
    pub fn new(filename: String) -> Result<Self, io::Error> {
        Ok(Self {
            current_path: "/".to_string(),
            file_system: FAT::new(filename)?
        })
    }
}

fn main() {
    println!("Hello, world!");
}
