pub mod app;
pub mod components;
pub mod config;
pub mod core;
pub mod input;
pub mod windows;

use app::App;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = ratatui::init();

    // Create app and run it
    let mut app = App::new();

    // Check if an argument is passed
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
      let path = std::path::Path::new(&args[1])
          .canonicalize()
          .unwrap_or(std::path::PathBuf::from(&args[1]));

      if path.is_dir() {
          app.load_workspace(&path);
      } else {
          app.open_file(&path, false);
      }
  }

    let res = app.run(&mut terminal);

    ratatui::restore();

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}
