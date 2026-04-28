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

#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() -> Result<(), Box<dyn Error>> {
    let path = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA")?).join("ryp")
    } else {
        PathBuf::from(env::var("HOME")?).join(".ryp")
    };

    let (tx, rx) = crossbeam_channel::unbounded();

    if path.exists() {
        if fs::read_dir(&path.join("plugins"))
              .map(|mut entries| entries.next().is_some()) // Check if at least one file exists
              .unwrap_or(false) {


            rayon::ThreadPoolBuilder::new()
                .num_threads(3)
                .build_global()
                .unwrap();

            // Load in the lua plugins
            // We pass in the plugins, to minimize thrown away work, and minimize mistakes
            let _ = crate::plugin::plugin_main::load_plugins(
                  CompactString::from(
                      path.join("plugins")
                          .to_str()
                          .expect("Could not complete path")
                  ), tx);
        } else {
          //  drop(tx);
        }
    } else {
        fs::create_dir_all(&path)?;
        drop(tx);
    }

    let mut terminal = ratatui::init();

    // Create app and run it
    let mut app = App::new(rx);

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
