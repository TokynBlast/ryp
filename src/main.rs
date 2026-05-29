pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;
pub mod plugin;

use app::App;
use std::error::Error;
use std::io::stdin;
use std::path::PathBuf;
use std::process::exit;
use std::{env, fs};
use crossterm::{execute, event::{EnableFocusChange, DisableFocusChange}};
use std::process::Command;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// Spawns a new instance of the current executable with administrator privileges.
fn escalate_privileges() -> bool {
    let current_exe = match env::current_exe() {
        Ok(path) => path,
        Err(_) => return false,
    };

    // Filter out the '--admin' flag so the new instance doesn't loop infinitely,
    // but pass all other original user arguments forward!
    let forward_args: Vec<String> = env::args()
        .skip(1)
        .filter(|arg| arg != "--admin")
        .collect();

    #[cfg(windows)]
    {
        // On Windows, we use PowerShell's 'runAs' verb to trigger the UAC prompt popup
        let mut args_string = format!("& '{}'", current_exe.to_string_lossy());
        for arg in forward_args {
            args_string.push_str(&format!(" '{}'", arg));
        }

        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                &format!("Start-Process -Verb runAs -FilePath powershell -ArgumentList '-NoProfile', '-Command', \"{}\"", args_string)
            ])
            .status();

        status.map(|s| s.success()).unwrap_or(false)
    }

    #[cfg(unix)]
    {
        // On Linux/macOS, we look for 'sudo'. If it fails or isn't present, we try 'pkexec'
        // (which brings up a friendly GUI password prompt for desktop Linux users!)
        let child = Command::new("sudo")
            .arg(&current_exe)
            .args(&forward_args)
            .spawn()
            .or_else(|_| {
                // Fallback for desktop users without sudo configured or who prefer GUI prompts
                Command::new("pkexec")
                    .arg(&current_exe)
                    .args(&forward_args)
                    .spawn()
            });

        match child {
            Ok(mut process) => process.wait().map(|s| s.success()).unwrap_or(false),
            Err(_) => false,
        }
    }
}

fn check_once<'a>(seen: &mut std::collections::HashSet<&'a str>, flag: &'a str) {
    if !seen.insert(flag) {
        eprintln!("Error: '{}' can only be used once", flag);
        exit(1);
    }
}

fn ask_continue() -> bool {
    print!("Continue? [Y/n] ");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut input = String::new();
    stdin().read_line(&mut input).ok();
    let trimmed = input.trim().to_lowercase();
    trimmed.is_empty() || trimmed == "y" || trimmed == "yes"
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let mut target = PathBuf::from(".").canonicalize().unwrap();

    // Check for conflicts
    if args.contains(&String::from("--tui")) && args.contains(&String::from("--gui")) {
        eprintln!("Error: --gui and --tui are mutually exclusive");
        exit(1);
    }

    // TODO: Make it so that it checks for doubles before running
    let mut seen: std::collections::HashSet<&str> =
        std::collections::HashSet::new();

    for arg in args.iter().skip(1) {
        // Match directly on the exact string slice to avoid prefix confusion
        match arg.as_str() {
            "--help" => {
                check_once(&mut seen, "--help");
                include!(concat!(env!("OUT_DIR"), "/usage.rs"));
                exit(0);
            }
            "--version" => {
                check_once(&mut seen, "--version");
                println!("{}", VERSION);
                exit(0);
            }
            "--admin" => {
                check_once(&mut seen, "--admin");
                escalate_privileges();
                exit(0);
            }
            "--wait" => {
                stdin().read_line(&mut String::new()).ok();
            }
            "--question" => {
                if !ask_continue() { exit(0); }
            }
            "--gui" | "--tui" => {}
            s if s.starts_with("--") => {
                eprintln!("Error: Unknown argument '{}'", s);
                exit(1);
            }
            s if s.starts_with('-') => {
                for chr in s[1..].chars() {
                    match chr {
                        'v' => {
                            check_once(&mut seen, "-v");
                            println!("{}", VERSION);
                        }
                        'h' => {
                            check_once(&mut seen, "-h");
                            include!(concat!(env!("OUT_DIR"), "/usage.rs"));
                            exit(0);
                        }
                        'a' => {
                            check_once(&mut seen, "-a");
                            escalate_privileges();
                            exit(0);
                        }
                        'w' => {
                            stdin().read_line(&mut String::new()).ok();
                        }
                        'q' => {
                            if !ask_continue() { exit(0); }
                        }
                        _ => {
                            eprintln!("Error: Unknown flag '{}'", chr);
                            exit(1);
                        }
                    }
                }
            }
            _ => target = PathBuf::from(arg).canonicalize().unwrap(),
        }
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
