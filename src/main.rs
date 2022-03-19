extern crate notify;
mod detector;
mod dictionary;

use std::sync::mpsc::channel;
use std::time::Duration;
use clap::{IntoApp, Parser, Subcommand};
use std::io::{Read, Write};
use std::path::Path;
use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};

/// Typo detector for english words
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// File path to read
    file_path: Option<String>,

    /// Suggest fix for typo
    #[clap(long, short)]
    suggest: bool,

    /// Watch file changes
    #[clap(long, short)]
    watch: bool,

    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Add custom word to dictionary
    Add {
        /// Add english word
        #[clap(default_value_t = String::from(""))]
        word: String,

        /// Display my dictionary
        #[clap(long, short)]
        dictionary: bool,
    },
}

fn main() {
    let args = Args::parse();
    match &args.command {
        Some(Commands::Add { word, dictionary }) => {
            match home::home_dir() {
                Some(mut path) => {
                    path.push(".config");
                    path.push("tpd");
                    path.push("dictionary");

                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .read(true)
                        .create(true)
                        .append(true)
                        .open(path)
                        .unwrap();

                    if *dictionary {
                        let mut data = String::new();
                        if let Ok(_) = file.read_to_string(&mut data) {
                            println!("{}", data);
                        } else {
                            println!("Unable to read dictionary");
                        }
                    } else {
                        if word.len() >= 1 {
                            write!(file, "{}\n", word).unwrap();
                            println!("Success store '{}' to dictionary!", word);
                        } else {
                            let mut app = Args::into_app();
                            app.print_help().unwrap();
                        }
                    }
                }
                None => {}
            }

            std::process::exit(0);
        }
        None => {}
    }

    match args.file_path.as_deref() {
        Some(path) => {
            watch(path, args.watch, args.suggest).unwrap()
        },
        _ => {
            let mut app = Args::into_app();
            app.print_help().unwrap();
        }
    }
}

fn watch(file_path: &str, should_watch: bool, suggest: bool) -> notify::Result<()> {
    detector::detect_typo(Path::new(file_path).to_path_buf(), suggest);
    if !should_watch {
        return Ok(());
    }

    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(300))?;
    watcher.watch(Path::new(&file_path), RecursiveMode::NonRecursive)?;

    loop {
        let event = match rx.recv() {
            Ok(event) => event,
            Err(err) => {
                println!("Config watcher channel dropped unexpectedly: {}", err);
                break;
            }
        };

        match event {
            DebouncedEvent::Rename(_, path)
            | DebouncedEvent::Write(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Chmod(path) => {
                println!("Processing file changes!");
                detector::detect_typo(path, suggest)
            }
            _ => (),
        }
    }
    Ok(())
}