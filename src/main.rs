pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;
pub mod plugin;

use app::App;
use compact_str::CompactString;
use jemallocator::Jemalloc;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::{env, fs};
use crossbeam;

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<(), Box<dyn Error>> {
    let path = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA")?).join("ryp")
    } else {
        PathBuf::from(env::var("HOME")?).join(".ryp")
    };
        fs::create_dir_all(&path)?;
    }

    let mut terminal = ratatui::init();

    let (tx, rx) = crossbeam::channel::unbounded();

    // Create app and run it
    let mut app = App::new(rx);

    // Load in the lua plugins
    // TODO: Only load this if there are active plugins to load :)
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
