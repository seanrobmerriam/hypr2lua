//! # hypr2lua
//!
//! A command-line tool that converts Hyprland hyprlang `.conf` files to the
//! Lua configuration format introduced in Hyprland v0.55+.
//!
//! ## Usage
//!
//! ```bash
//! hypr2lua <input.conf> [output.lua]
//! hypr2lua --batch <directory>
//! ```
//!
//! If no output path is specified, the output will be written to a `.lua` file
//! with the same base name as the input.
//!
//! The `--batch` flag converts all `.conf` files in a directory to `.lua` files.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

use hypr2lua::{generate, parse};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: hypr2lua <input.conf> [output.lua]");
        eprintln!("       hypr2lua --batch <directory>");
        eprintln!("  Converts Hyprlang .conf files to Hyprland .lua format");
        process::exit(1);
    }

    if args[1] == "--batch" || args[1] == "-b" {
        if args.len() < 3 {
            eprintln!("Usage: hypr2lua --batch <directory>");
            process::exit(1);
        }
        batch_convert(&args[2]);
        return;
    }

    convert_single(&args[1], args.get(2).map(|s| s.as_str()));
}

fn convert_single(input_path: &str, output_path: Option<&str>) {
    let input_path = PathBuf::from(input_path);
    let output_path = if let Some(p) = output_path {
        PathBuf::from(p)
    } else {
        input_path.with_extension("lua")
    };

    let input = match fs::read_to_string(&input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {}: {}", input_path.display(), e);
            process::exit(1);
        }
    };

    let ast = parse(&input);
    let lua_output = generate(&ast);

    match fs::write(&output_path, &lua_output) {
        Ok(()) => println!("Converted {} -> {}", input_path.display(), output_path.display()),
        Err(e) => {
            eprintln!("Error writing {}: {}", output_path.display(), e);
            process::exit(1);
        }
    }
}

fn batch_convert(dir_path: &str) {
    let dir = PathBuf::from(dir_path);
    
    if !dir.is_dir() {
        eprintln!("Error: {} is not a directory", dir.display());
        process::exit(1);
    }

    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading directory {}: {}", dir.display(), e);
            process::exit(1);
        }
    };

    let mut converted = 0;
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                eprintln!("Error reading entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("conf") {
            let output_path = path.with_extension("lua");
            let input = match fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading {}: {}", path.display(), e);
                    continue;
                }
            };

            let ast = parse(&input);
            let lua_output = generate(&ast);

            match fs::write(&output_path, &lua_output) {
                Ok(()) => {
                    println!("Converted {} -> {}", path.display(), output_path.display());
                    converted += 1;
                }
                Err(e) => {
                    eprintln!("Error writing {}: {}", output_path.display(), e);
                }
            }
        }
    }

    println!("Batch conversion complete: {} files converted", converted);
}
