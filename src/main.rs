mod app;

use app::core::App;
use std::{
    env,
    io::{self},
    path::PathBuf,
};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut start_path: PathBuf = PathBuf::new();

    if args.len() > 1 {
        let nav_cmd = app::pathmanager::convert_path_to_nav(args[args.len() - 1].as_str())?;
        start_path = app::pathmanager::resolve_path(&start_path, &nav_cmd)?;
    }

    let mut app = App::new(&start_path)?;

    let r = app.run();

    if let Err(e) = r {
        eprintln!("Error: {}", e);
        return Err(e);
    }

    app.end()?;

    Ok(())
}
