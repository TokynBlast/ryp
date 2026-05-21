pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;
pub mod plugin;

use app::App;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::{env, fs};
use crossterm::{execute, event::{EnableFocusChange, DisableFocusChange}};

#[cfg(target_os = "uefi")]
#[no_std]

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() -> Result<(), Box<dyn Error>> {
    // Check if an argument is passed
    let args: Vec<String> = std::env::args().collect();
    let target: PathBuf;

    if let Some(arg) = args.get(1) {
        if let Some(flag) = arg.strip_prefix("--") {
            match flag {
                "help" => {
                    println!("Ryp is a text editor. By default, it will open to the current directory.\n--help - prints help text\n--version - prints the version");
                    exit(0);
                },
                "version" => {
                    println!("0.1.0");
                    exit(0);
                },
                _ => panic!("Info getter does not expect any extra arguments."),
            }
        } else {
            target = Path::new(arg).canonicalize().unwrap();
        }
    } else {
        target = PathBuf::from(".").canonicalize().unwrap();
    }

    reqwest::Client::new();

    let path = if cfg!(windows) {
        PathBuf::from(env::var("APPDATA")?).join("ryp")
    } else {
        // This is a standard config location, and we can store plugins here too,
        // so we can just delete our config folder and our application on uninstall
        PathBuf::from(env::var("HOME")?).join(".config").join("ryp")
    };

    let (plugin_tx, plugin_rx) = crossbeam_channel::unbounded();

    if path.exists() {
        if path.join("config").exists() {
            if fs::read_dir(&path.join("plugins"))
                  .map(|mut entries| entries.next().is_some()) // Check if at least one file exists
                  .unwrap_or(false) {


                rayon::ThreadPoolBuilder::new()
                    .num_threads(3)
                    .build_global()
                    .unwrap();

                // Load in the lua plugins
                // We pass in the plugins, to minimize thrown away work, and minimize mistakes
                let _ = crate::plugin::plugin_main::load_plugins(path.join("plugins"), plugin_tx);
            } else {
                drop(plugin_tx);
            }
        } else {
            rayon::ThreadPoolBuilder::new()
                .num_threads(2)
                .build_global()
                .unwrap();
            fs::create_dir_all(&path.join("plugins"))?;
            drop(plugin_tx);
        }
    } else {
        fs::create_dir_all(&path.join("plugins"))?;
        drop(plugin_tx);
    }

    if !fs::exists(&path.join("config.json")).unwrap_or(false) {
        fs::File::create(&path.join("config.json"))?;
    }

    execute!(std::io::stdout(), EnableFocusChange)?;

    let mut terminal = ratatui::init();

    // Create app and run it
    let mut app = App::new(plugin_rx);

    if target.is_dir() {
        app.load_workspace(&target);
    } else {
        app.open_file(&target, false);
    }

    let res = app.run(&mut terminal);

    ratatui::restore();

    execute!(std::io::stdout(), DisableFocusChange)?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
