//! # FracturedJson
//!
//! A JSON formatter that produces human-readable output with smart line breaks,
//! table-like alignment, and optional comment support.
//!
//! FracturedJson formats JSON data in a way that's easy for humans to read while
//! remaining fairly compact:
//!
//! - Arrays and objects are written on single lines when they're short and simple enough
//! - When several lines have similar structure, their fields are aligned like a table
//! - Long arrays are written with multiple items per line
//! - Comments (non-standard JSON) can be preserved if enabled
//!
//! ## Command-Line Tool
//!
//! This crate includes the `fjson` CLI tool for formatting JSON from the terminal:
//!
//! ```sh
//! # Install
//! cargo install fracturedjson
//!
//! # Format JSON from stdin
//! echo '{"a":1,"b":2}' | fjson
//!
//! # Format a file
//! fjson input.json -o output.json
//!
//! # Minify
//! fjson --compact < input.json
//! ```
//!
//! Run `fjson --help` for all options.
//!
//! ## Quick Start
//!
//! ```rust
//! use fracturedjson::Formatter;
//!
//! let input = r#"{"name":"Alice","scores":[95,87,92],"active":true}"#;
//!
//! let mut formatter = Formatter::new();
//! let output = formatter.reformat(input, 0).unwrap();
//!
//! println!("{}", output);
//! ```
//!
//! ## Serializing Rust Types
//!
//! Any type implementing [`serde::Serialize`] can be formatted directly:
//!
//! ```rust
//! use fracturedjson::Formatter;
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Player {
//!     name: String,
//!     scores: Vec<i32>,
//! }
//!
//! let player = Player {
//!     name: "Alice".into(),
//!     scores: vec![95, 87, 92],
//! };
//!
//! let mut formatter = Formatter::new();
//! let output = formatter.serialize(&player, 0, 100).unwrap();
//! ```
//!
//! ## Configuration
//!
//! Customize formatting behavior through [`FracturedJsonOptions`]:
//!
//! ```rust
//! use fracturedjson::{Formatter, EolStyle, NumberListAlignment};
//!
//! let mut formatter = Formatter::new();
//! formatter.options.max_total_line_length = 80;
//! formatter.options.indent_spaces = 2;
//! formatter.options.json_eol_style = EolStyle::Lf;
//! formatter.options.number_list_alignment = NumberListAlignment::Decimal;
//!
//! let output = formatter.reformat(r#"{"values":[1,2,3]}"#, 0).unwrap();
//! ```
//!
//! ## Comment Support
//!
//! FracturedJson can handle JSON with comments (non-standard) when enabled:
//!
//! ```rust
//! use fracturedjson::{Formatter, CommentPolicy};
//!
//! let input = r#"{
//!     // This is a comment
//!     "name": "Alice"
//! }"#;
//!
//! let mut formatter = Formatter::new();
//! formatter.options.comment_policy = CommentPolicy::Preserve;
//!
//! let output = formatter.reformat(input, 0).unwrap();
//! ```
//!
//! ## Example Output
//!
//! Given appropriate input, FracturedJson produces output like:
//!
//! ```json
//! {
//!     "SimilarObjects": [
//!         { "type": "turret",    "hp": 400, "loc": {"x": 47, "y":  -4} },
//!         { "type": "assassin",  "hp":  80, "loc": {"x": 12, "y":   6} },
//!         { "type": "berserker", "hp": 150, "loc": {"x":  0, "y":   0} }
//!     ]
//! }
//! ```
//!
//! Notice how:
//! - Similar objects are aligned in a table format
//! - Numbers are right-aligned within their columns
//! - The structure remains compact while being highly readable

mod buffer;
mod convert;
mod error;
mod formatter;
mod model;
mod options;
mod parser;
mod table_template;
mod tokenizer;

pub use crate::error::FracturedJsonError;
pub use crate::formatter::Formatter;
pub use crate::model::{InputPosition, JsonItemType};
pub use crate::options::{
    CommentPolicy, EolStyle, FracturedJsonOptions, NumberListAlignment, TableCommaPlacement,
};
