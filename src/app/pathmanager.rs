use std::{
    env, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum NavigationCommand {
    Up(usize), // Naik ke direktori induk sebanyak n level
    Root,      // Kembali ke root direktori
    Home,      // Kembali ke home direktori
    WorkingDirectory,
    Absolute(PathBuf), // Path absolut
    Relative(PathBuf), // Path relatif
}

pub fn convert_path_to_nav(path: &str) -> io::Result<NavigationCommand> {
    match path {
        s if s.starts_with("..") => {
            let levels = s.matches('.').count().wrapping_sub(1);
            Ok(NavigationCommand::Up(levels))
        }

        "." => Ok(NavigationCommand::WorkingDirectory),
        "/" => Ok(NavigationCommand::Root),
        "~" | "$HOME" => Ok(NavigationCommand::Home),

        s if s.starts_with('/') => Ok(NavigationCommand::Absolute(PathBuf::from(s))),
        s => Ok(NavigationCommand::Relative(PathBuf::from(s))),
    }
}

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            std::path::Component::Normal(c) => normalized.push(c),
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            std::path::Component::RootDir => normalized.push("/"),
            _ => {}
        }
    }

    normalized
}

pub fn resolve_path(current_dir: &Path, command: &NavigationCommand) -> io::Result<PathBuf> {
    match command {
        // Naik ke direktori induk
        NavigationCommand::Up(levels) => {
            let mut target = current_dir.to_path_buf();
            for _ in 0..*levels {
                if let Some(parent) = target.parent() {
                    target = parent.to_path_buf();
                } else {
                    break;
                }
            }
            Ok(target)
        }

        NavigationCommand::WorkingDirectory => Ok(env::current_dir().unwrap_or(PathBuf::from("/"))),

        NavigationCommand::Root => Ok(PathBuf::from("/")),

        NavigationCommand::Home => {
            let home = env::var("HOME")
                .map_err(|_| io::Error::new(io::ErrorKind::NotFound, "Home directory not found"))?;
            Ok(PathBuf::from(home))
        }

        NavigationCommand::Absolute(path) => Ok(path.clone()),

        NavigationCommand::Relative(path) => {
            let expanded_path = if path.starts_with("~") {
                let home = env::var("HOME").map_err(|_| {
                    io::Error::new(io::ErrorKind::NotFound, "Home directory not found")
                })?;
                PathBuf::from(home).join(path.strip_prefix("~").unwrap())
            } else {
                path.clone()
            };

            let resolved_path = if expanded_path.is_absolute() {
                expanded_path.clone()
            } else {
                current_dir.join(expanded_path)
            };

            Ok(normalize_path(&resolved_path))
        }
    }
}
