/// Line ending style for the formatted output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EolStyle {
    /// Windows-style line endings (`\r\n`).
    Crlf,
    /// Unix-style line endings (`\n`).
    Lf,
}

/// Policy for handling comments in JSON input.
///
/// Standard JSON does not support comments, but many JSON-like formats
/// (such as JSONC used by VS Code) do allow them. This enum controls
/// how comments are handled during formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentPolicy {
    /// Return an error if comments are encountered in the input.
    /// This is the default, enforcing strict JSON compliance.
    TreatAsError,
    /// Silently remove any comments from the output.
    Remove,
    /// Keep comments in the output, preserving their relative positions.
    Preserve,
}

/// Alignment style for numbers in arrays formatted as tables.
///
/// When arrays of numbers are formatted across multiple lines,
/// this setting controls how the numbers are aligned within their columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberListAlignment {
    /// Align numbers to the left of their column.
    Left,
    /// Align numbers to the right of their column.
    Right,
    /// Align numbers by their decimal point (or implied decimal for integers).
    /// This is often the most readable option for mixed integer/decimal data.
    Decimal,
    /// Normalize numbers to a consistent format and align by decimal point.
    Normalize,
}

/// Controls where commas are placed relative to padding in table-formatted output.
///
/// When objects or arrays are formatted in a table layout with aligned columns,
/// this setting determines whether commas appear before or after the padding
/// spaces used for alignment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableCommaPlacement {
    /// Place commas immediately after values, before any padding.
    /// Example: `"name",    "value"`
    BeforePadding,
    /// Place commas after padding, at the end of the padded column.
    /// Example: `"name"    ,"value"`
    AfterPadding,
    /// Place commas before padding for most values, but after padding for numbers.
    /// This often produces the cleanest-looking output for mixed data.
    BeforePaddingExceptNumbers,
}

/// Configuration options for JSON formatting.
///
/// This struct contains all settings that control how JSON is formatted.
/// Use [`Default::default()`] or [`FracturedJsonOptions::recommended()`]
/// to get sensible defaults, then modify individual fields as needed.
///
/// # Example
///
/// ```rust
/// use fracturedjson::{FracturedJsonOptions, EolStyle, CommentPolicy};
///
/// let mut options = FracturedJsonOptions::default();
/// options.max_total_line_length = 80;
/// options.indent_spaces = 2;
/// options.comment_policy = CommentPolicy::Preserve;
/// ```
#[derive(Debug, Clone)]
pub struct FracturedJsonOptions {
    /// Line ending style for the output. Default: [`EolStyle::Lf`].
    pub json_eol_style: EolStyle,

    /// Maximum length of a line before it's broken into multiple lines.
    /// Default: 120.
    pub max_total_line_length: usize,

    /// Maximum nesting depth for arrays/objects to be written on a single line.
    /// A value of 0 means only primitive values can be inlined.
    /// A value of 1 allows simple arrays/objects with primitive elements.
    /// Set to -1 to disable inline formatting entirely.
    /// Default: 2.
    pub max_inline_complexity: isize,

    /// Maximum nesting depth for arrays to use compact multi-line formatting
    /// (multiple items per line). Set to -1 to disable.
    /// Default: 2.
    pub max_compact_array_complexity: isize,

    /// Maximum nesting depth for arrays/objects to be formatted as aligned tables.
    /// Set to -1 to disable table formatting.
    /// Default: 2.
    pub max_table_row_complexity: isize,

    /// Maximum number of spaces to use for property name padding in table format.
    /// If aligning property names would require more padding than this, alignment
    /// is skipped for that container.
    /// Default: 16.
    pub max_prop_name_padding: usize,

    /// If true, the colon comes before the property name padding.
    /// Example with true: `"a": 1` vs `"aaa": 2`
    /// Example with false: `"a"  : 1` vs `"aaa": 2`
    /// Default: false.
    pub colon_before_prop_name_padding: bool,

    /// Where to place commas in table-formatted output.
    /// Default: [`TableCommaPlacement::BeforePaddingExceptNumbers`].
    pub table_comma_placement: TableCommaPlacement,

    /// Minimum number of items required per row when formatting arrays
    /// in compact multi-line mode. Default: 3.
    pub min_compact_array_row_items: usize,

    /// Depth at which containers are always expanded (never inlined).
    /// Containers at this depth or shallower will always be multi-line.
    /// Set to -1 to disable (allow inlining at any depth).
    /// Default: -1.
    pub always_expand_depth: isize,

    /// Add spaces inside brackets for nested containers: `[ [1, 2] ]` vs `[[1, 2]]`.
    /// Default: true.
    pub nested_bracket_padding: bool,

    /// Add spaces inside brackets for simple (non-nested) containers: `[ 1, 2 ]` vs `[1, 2]`.
    /// Default: false.
    pub simple_bracket_padding: bool,

    /// Add a space after colons in objects: `"key": value` vs `"key":value`.
    /// Default: true.
    pub colon_padding: bool,

    /// Add a space after commas: `[1, 2, 3]` vs `[1,2,3]`.
    /// Default: true.
    pub comma_padding: bool,

    /// Add a space before comments: `value /*comment*/` vs `value/*comment*/`.
    /// Default: true.
    pub comment_padding: bool,

    /// Alignment style for numbers in array tables.
    /// Default: [`NumberListAlignment::Decimal`].
    pub number_list_alignment: NumberListAlignment,

    /// Number of spaces per indentation level. Ignored if `use_tab_to_indent` is true.
    /// Default: 4.
    pub indent_spaces: usize,

    /// Use tabs instead of spaces for indentation.
    /// Default: false.
    pub use_tab_to_indent: bool,

    /// A string to prepend to every line of output. Useful for embedding
    /// formatted JSON within other content.
    /// Default: empty string.
    pub prefix_string: String,

    /// How to handle comments in the input.
    /// Default: [`CommentPolicy::TreatAsError`].
    pub comment_policy: CommentPolicy,

    /// Preserve blank lines from the input in the output.
    /// Only meaningful when `comment_policy` is not `TreatAsError`.
    /// Default: false.
    pub preserve_blank_lines: bool,

    /// Allow trailing commas in the input (non-standard JSON).
    /// Default: false.
    pub allow_trailing_commas: bool,
}

impl Default for FracturedJsonOptions {
    fn default() -> Self {
        Self {
            json_eol_style: EolStyle::Lf,
            max_total_line_length: 120,
            max_inline_complexity: 2,
            max_compact_array_complexity: 2,
            max_table_row_complexity: 2,
            max_prop_name_padding: 16,
            colon_before_prop_name_padding: false,
            table_comma_placement: TableCommaPlacement::BeforePaddingExceptNumbers,
            min_compact_array_row_items: 3,
            always_expand_depth: -1,
            nested_bracket_padding: true,
            simple_bracket_padding: false,
            colon_padding: true,
            comma_padding: true,
            comment_padding: true,
            number_list_alignment: NumberListAlignment::Decimal,
            indent_spaces: 4,
            use_tab_to_indent: false,
            prefix_string: String::new(),
            comment_policy: CommentPolicy::TreatAsError,
            preserve_blank_lines: false,
            allow_trailing_commas: false,
        }
    }
}

impl FracturedJsonOptions {
    /// Creates a new `FracturedJsonOptions` with recommended settings.
    ///
    /// Currently identical to [`Default::default()`], but may include
    /// improved defaults in future versions without breaking compatibility.
    pub fn recommended() -> Self {
        Self::default()
    }
}
