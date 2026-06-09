//! # hypr2lua
//!
//! A library and CLI tool for converting Hyprland hyprlang `.conf` files to
//! the Lua configuration format introduced in Hyprland v0.55+.
//!
//! ## Library Usage
//!
//! ```rust
//! use hypr2lua::parser::parse;
//! use hypr2lua::lua_gen::generate;
//!
//! let conf = r#"
//! $mod = SUPER
//!
//! general {
//!     gaps_in = 5
//! }
//!
//! bind = $mod, Q, exec, foot
//! "#;
//!
//! let nodes = parse(conf);
//! let lua = generate(&nodes);
//! println!("{}", lua);
//! ```

pub mod lua_gen;
pub mod parser;

pub use lua_gen::generate;
pub use parser::{parse, Node};
