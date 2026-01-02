use std::fmt::{self, Display};

use crate::model::InputPosition;

#[derive(Debug, Clone)]
pub struct FracturedJsonError {
    pub message: String,
    pub input_position: Option<InputPosition>,
}

impl FracturedJsonError {
    pub fn new(message: impl Into<String>, pos: Option<InputPosition>) -> Self {
        let message = message.into();
        let message = if let Some(p) = pos {
            format!("{} at idx={}, row={}, col={}", message, p.index, p.row, p.column)
        } else {
            message
        };
        Self { message, input_position: pos }
    }

    pub fn simple(message: impl Into<String>) -> Self {
        Self::new(message, None)
    }
}

impl Display for FracturedJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for FracturedJsonError {}
