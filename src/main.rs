use std::{error::Error, io};

use fat::FAT;

mod cli;
mod fat;
mod units;

pub struct Application {
    running: bool,
    current_path: String,
    file_system: FAT,
}

impl Application {
    pub fn new(filename: String) -> Result<Self, io::Error> {
        Ok(Self {
            running: true,
            current_path: "/".to_string(),
            file_system: FAT::new(filename)?,
        })
    }

    pub fn running(&self) -> bool {
        self.running
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let filename = std::env::args().nth(1).expect("Please provide a file!");
    let mut app = Application::new(filename)?;

    while app.running() {
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;

        let trimmed = line.trim();
        if trimmed.len() == 0 {
            continue;
        }

        if let Some(handler) = cli::get(line.trim()) {
            if let Err(err) = handler.handle(&mut app) {
                println!("{}", err);
            } else {
                println!("OK");
            }
        } else {
            println!("invalid command: {}", trimmed);
        }
    }

    Ok(())
}
