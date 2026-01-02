use std::fmt::{self, Display};

use crate::model::InputPosition;

/// Error type returned by formatting operations.
///
/// This error is returned when JSON parsing fails (invalid syntax, unexpected tokens)
/// or when formatting constraints are violated (recursion limit exceeded, etc.).
///
/// When the error is associated with a specific location in the input, the
/// `input_position` field will contain the position information, and the
/// message will include human-readable location details.
///
/// # Example
///
/// ```rust
/// use fracturedjson::Formatter;
///
/// let mut formatter = Formatter::new();
/// let result = formatter.reformat("{ invalid json }", 0);
///
/// match result {
///     Ok(_) => println!("Success"),
///     Err(e) => {
///         println!("Error: {}", e);
///         if let Some(pos) = e.input_position {
///             println!("At row {}, column {}", pos.row, pos.column);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FracturedJsonError {
    /// The error message, including position information if available.
    pub message: String,

    /// The position in the input where the error occurred, if applicable.
    pub input_position: Option<InputPosition>,
}

impl FracturedJsonError {
    /// Creates a new error with an optional input position.
    ///
    /// If a position is provided, it will be appended to the message
    /// in a human-readable format.
    pub fn new(message: impl Into<String>, pos: Option<InputPosition>) -> Self {
        let message = message.into();
        let message = if let Some(p) = pos {
            format!(
                "{} at idx={}, row={}, col={}",
                message, p.index, p.row, p.column
            )
        } else {
            message
        };
        Self {
            message,
            input_position: pos,
        }
    }

    /// Creates a new error without position information.
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
