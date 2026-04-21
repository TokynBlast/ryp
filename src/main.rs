pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;
pub mod plugin;

use app::App;
use jemallocator::Jemalloc;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use crossbeam;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<(), Box<dyn Error>> {
    let path = if cfg!(windows) {
        // Rust won't expand %APPDATA%, so we get the variable
        let base = env::var("APPDATA")?;
        PathBuf::from(base).join("ryp")
    } else {
        // Get /home/user from $HOME
        let base = env::var("HOME")?;
        PathBuf::from(base).join(".ryp")
    };

    if !path.exists() {
        fs::create_dir_all(&path)?;
    }

    let mut terminal = ratatui::init();

    let (tx, rx) = crossbeam::channel::unbounded();

    // Create app and run it
    let mut app = App::new(rx);

    // Load in the lua plugins
    let _ = crate::plugin::plugin_main::load_plugins(tx);

    // Check if an argument is passed
    let args: Vec<String> = std::env::args().collect(); // Would benifit slightly from compact strings
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
