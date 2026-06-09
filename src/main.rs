//! # hypr2lua
//!
//! A command-line tool that converts Hyprland hyprlang `.conf` files to the
//! Lua configuration format introduced in Hyprland v0.55+.
//!
//! ## Usage
//!
//! ```bash
//! hypr2lua <input.conf> [output.lua]
//! ```
//!
//! If no output path is specified, the output will be written to a `.lua` file
//! with the same base name as the input.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

mod parser;
mod lua_gen;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: hypr2lua <input.conf> [output.lua]");
        eprintln!("  Converts a Hyprlang .conf file to Hyprland .lua format");
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
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

    let ast = parser::parse(&input);
    let lua_output = lua_gen::generate(&ast);

    match fs::write(&output_path, &lua_output) {
        Ok(()) => println!("Converted {} -> {}", input_path.display(), output_path.display()),
        Err(e) => {
            eprintln!("Error writing {}: {}", output_path.display(), e);
            process::exit(1);
        }
    }
}
