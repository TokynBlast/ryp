pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;

use app::App;
use std::error::Error;
use std::path::{Path, PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    // Create app and run it
    let mut app = App::new();

    // Check if an argument is passed
    let args: Vec<String> = std::env::args().collect();
    let target = if args.len() > 1 {
        Path::new(&args[1]).canonicalize().unwrap_or(PathBuf::from(&args[1]))
    } else {
        PathBuf::from(".")
    };

    if target.is_dir() {
        app.load_workspace(&target);
    } else {
        app.open_file(&target, false);
    }

    let res = app.run(&mut terminal);

    ratatui::restore();

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
