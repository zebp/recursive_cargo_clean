use std::ffi::OsStr;
use std::io::Result;
use std::path::*;
use std::process::Command;

use clap::App;
use colored::Colorize;
use jwalk::WalkDir;

fn main() {
    let yaml = clap::load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let root = matches.value_of("root").map(Path::new).unwrap();
    let max_depth = matches.value_of("max-depth").map(|max_depth| {
        max_depth
            .parse::<usize>()
            .expect("unable to parse max depth")
    });

    for project in find_cargo_directories(root, max_depth) {
        match project {
            Ok(project) => {
                if let Err(error) = clean_cargo_project(&project) {
                    eprintln!("Error while cleaning project: {:?}", error);
                } else {
                    let project_name = project
                        .file_name()
                        .map(OsStr::to_str)
                        .flatten()
                        .map(|name| name.green())
                        .unwrap_or("unknown name".dimmed());

                    println!("Cleaned project {}.", project_name);
                }
            }
            Err(error) => eprintln!("Error while scanning for projects: {:?}", error),
        }
    }
}

fn find_cargo_directories(root: &Path, depth: Option<usize>) -> Vec<Result<PathBuf>> {
    WalkDir::new(root)
        .max_depth(depth.unwrap_or(usize::max_value()))
        .num_threads(num_cpus::get()) // I assumed this line wouldn't be required but adding it gave a drastic improvement in performance
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry) if is_cargo_project(&entry.path()) => Some(Ok(entry.path())),
            Ok(_) => None,
            Err(error) => Some(Err(error)),
        })
        .collect()
}

fn clean_cargo_project(directory: &Path) -> Result<()> {
    Command::new("cargo")
        .arg("clean")
        .current_dir(directory)
        .output()
        .map(|_| ())
}

fn is_cargo_project(path: &Path) -> bool {
    path.is_dir() && path.join("Cargo.toml").exists()
}
