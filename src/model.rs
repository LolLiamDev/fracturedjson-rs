/// The type of a JSON element.
///
/// This enum represents the different types of items that can appear in JSON,
/// including standard JSON types (null, boolean, string, number, object, array)
/// and extended types for comment support (blank lines, comments).
///
/// This is primarily exposed for advanced use cases where you need to inspect
/// the parsed structure. Most users won't need to interact with this directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonItemType {
    /// JSON `null` value.
    Null,
    /// JSON `false` boolean.
    False,
    /// JSON `true` boolean.
    True,
    /// A JSON string value.
    String,
    /// A JSON number value.
    Number,
    /// A JSON object (`{}`).
    Object,
    /// A JSON array (`[]`).
    Array,
    /// A blank line (when `preserve_blank_lines` is enabled).
    BlankLine,
    /// A line comment (`// ...`).
    LineComment,
    /// A block comment (`/* ... */`).
    BlockComment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    BeginArray,
    EndArray,
    BeginObject,
    EndObject,
    String,
    Number,
    Null,
    True,
    False,
    BlockComment,
    LineComment,
    BlankLine,
    Comma,
    Colon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BracketPaddingType {
    Empty = 0,
    Simple = 1,
    Complex = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableColumnType {
    Unknown,
    Simple,
    Number,
    Array,
    Object,
    Mixed,
}

/// A position within the JSON input text.
///
/// Used to report the location of errors or elements within the source.
/// All values are zero-indexed.
///
/// # Example
///
/// ```rust
/// use fracturedjson::InputPosition;
///
/// // Represents position at the start of line 3, column 5
/// let pos = InputPosition { index: 42, row: 2, column: 4 };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputPosition {
    /// Byte offset from the start of the input (zero-indexed).
    pub index: usize,
    /// Line number (zero-indexed, so first line is 0).
    pub row: usize,
    /// Column number within the line (zero-indexed).
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonToken {
    pub token_type: TokenType,
    pub text: String,
    pub input_position: InputPosition,
}

#[derive(Debug, Clone)]
pub struct JsonItem {
    pub item_type: JsonItemType,
    pub input_position: InputPosition,
    pub complexity: usize,
    pub name: String,
    pub value: String,
    pub prefix_comment: String,
    pub middle_comment: String,
    pub middle_comment_has_new_line: bool,
    pub postfix_comment: String,
    pub is_post_comment_line_style: bool,
    pub name_length: usize,
    pub value_length: usize,
    pub prefix_comment_length: usize,
    pub middle_comment_length: usize,
    pub postfix_comment_length: usize,
    pub minimum_total_length: usize,
    pub requires_multiple_lines: bool,
    pub children: Vec<JsonItem>,
}

impl Default for JsonItem {
    fn default() -> Self {
        Self {
            item_type: JsonItemType::Null,
            input_position: InputPosition {
                index: 0,
                row: 0,
                column: 0,
            },
            complexity: 0,
            name: String::new(),
            value: String::new(),
            prefix_comment: String::new(),
            middle_comment: String::new(),
            middle_comment_has_new_line: false,
            postfix_comment: String::new(),
            is_post_comment_line_style: false,
            name_length: 0,
            value_length: 0,
            prefix_comment_length: 0,
            middle_comment_length: 0,
            postfix_comment_length: 0,
            minimum_total_length: 0,
            requires_multiple_lines: false,
            children: Vec::new(),
        }
    }
}
