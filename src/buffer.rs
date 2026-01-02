use crate::model::{BracketPaddingType, JsonItemType};
use crate::options::{EolStyle, FracturedJsonOptions};

#[derive(Debug, Default)]
pub struct StringJoinBuffer {
    line_buff: Vec<String>,
    doc_buff: Vec<String>,
}

impl StringJoinBuffer {
    pub fn add(&mut self, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.line_buff.push(value.to_string());
        }
        self
    }

    pub fn spaces(&mut self, count: usize) -> &mut Self {
        if count > 0 {
            self.line_buff.push(" ".repeat(count));
        }
        self
    }

    pub fn end_line(&mut self, eol: &str) -> &mut Self {
        self.add_line_to_writer(eol);
        self
    }

    pub fn flush(&mut self) -> &mut Self {
        self.add_line_to_writer("");
        self
    }

    pub fn as_string(&self) -> String {
        self.doc_buff.join("")
    }

    fn add_line_to_writer(&mut self, eol: &str) {
        if self.line_buff.is_empty() && eol.is_empty() {
            return;
        }

        let mut line = self.line_buff.join("");
        while line.ends_with(|c: char| c.is_whitespace()) {
            line.pop();
        }

        self.doc_buff.push(format!("{}{}", line, eol));
        self.line_buff.clear();
    }
}

#[derive(Clone, Debug)]
pub struct PaddedFormattingTokens {
    comma: String,
    colon: String,
    comment: String,
    eol: String,
    dummy_comma: String,
    comma_len: usize,
    colon_len: usize,
    comment_len: usize,
    literal_null_len: usize,
    literal_true_len: usize,
    literal_false_len: usize,
    prefix_string_len: usize,
    arr_start: Vec<String>,
    arr_end: Vec<String>,
    obj_start: Vec<String>,
    obj_end: Vec<String>,
    arr_start_len: Vec<usize>,
    arr_end_len: Vec<usize>,
    obj_start_len: Vec<usize>,
    obj_end_len: Vec<usize>,
    indent_strings: Vec<String>,
}

impl PaddedFormattingTokens {
    pub fn new(opts: &FracturedJsonOptions, str_len_func: &dyn Fn(&str) -> usize) -> Self {
        let mut arr_start = vec![String::new(); 3];
        arr_start[BracketPaddingType::Empty as usize] = "[".to_string();
        arr_start[BracketPaddingType::Simple as usize] = if opts.simple_bracket_padding { "[ " } else { "[" }.to_string();
        arr_start[BracketPaddingType::Complex as usize] = if opts.nested_bracket_padding { "[ " } else { "[" }.to_string();

        let mut arr_end = vec![String::new(); 3];
        arr_end[BracketPaddingType::Empty as usize] = "]".to_string();
        arr_end[BracketPaddingType::Simple as usize] = if opts.simple_bracket_padding { " ]" } else { "]" }.to_string();
        arr_end[BracketPaddingType::Complex as usize] = if opts.nested_bracket_padding { " ]" } else { "]" }.to_string();

        let mut obj_start = vec![String::new(); 3];
        obj_start[BracketPaddingType::Empty as usize] = "{".to_string();
        obj_start[BracketPaddingType::Simple as usize] = if opts.simple_bracket_padding { "{ " } else { "{" }.to_string();
        obj_start[BracketPaddingType::Complex as usize] = if opts.nested_bracket_padding { "{ " } else { "{" }.to_string();

        let mut obj_end = vec![String::new(); 3];
        obj_end[BracketPaddingType::Empty as usize] = "}".to_string();
        obj_end[BracketPaddingType::Simple as usize] = if opts.simple_bracket_padding { " }" } else { "}" }.to_string();
        obj_end[BracketPaddingType::Complex as usize] = if opts.nested_bracket_padding { " }" } else { "}" }.to_string();

        let comma = if opts.comma_padding { ", " } else { "," }.to_string();
        let colon = if opts.colon_padding { ": " } else { ":" }.to_string();
        let comment = if opts.comment_padding { " " } else { "" }.to_string();
        let eol = if opts.json_eol_style == EolStyle::Crlf { "\r\n" } else { "\n" }.to_string();

        let arr_start_len = arr_start.iter().map(|s| str_len_func(s)).collect();
        let arr_end_len = arr_end.iter().map(|s| str_len_func(s)).collect();
        let obj_start_len = obj_start.iter().map(|s| str_len_func(s)).collect();
        let obj_end_len = obj_end.iter().map(|s| str_len_func(s)).collect();

        let indent_strings = vec![
            String::new(),
            if opts.use_tab_to_indent { "\t".to_string() } else { " ".repeat(opts.indent_spaces) },
        ];

        let comma_len = str_len_func(&comma);
        let colon_len = str_len_func(&colon);
        let comment_len = str_len_func(&comment);
        let literal_null_len = str_len_func("null");
        let literal_true_len = str_len_func("true");
        let literal_false_len = str_len_func("false");
        let prefix_string_len = str_len_func(&opts.prefix_string);
        let dummy_comma = " ".repeat(comma_len);

        Self {
            comma,
            colon,
            comment,
            eol,
            dummy_comma,
            comma_len,
            colon_len,
            comment_len,
            literal_null_len,
            literal_true_len,
            literal_false_len,
            prefix_string_len,
            arr_start,
            arr_end,
            obj_start,
            obj_end,
            arr_start_len,
            arr_end_len,
            obj_start_len,
            obj_end_len,
            indent_strings,
        }
    }

    pub fn comma(&self) -> &str {
        &self.comma
    }
    pub fn colon(&self) -> &str {
        &self.colon
    }
    pub fn comment(&self) -> &str {
        &self.comment
    }
    pub fn eol(&self) -> &str {
        &self.eol
    }
    pub fn comma_len(&self) -> usize {
        self.comma_len
    }
    pub fn colon_len(&self) -> usize {
        self.colon_len
    }
    pub fn comment_len(&self) -> usize {
        self.comment_len
    }
    pub fn literal_null_len(&self) -> usize {
        self.literal_null_len
    }
    pub fn literal_true_len(&self) -> usize {
        self.literal_true_len
    }
    pub fn literal_false_len(&self) -> usize {
        self.literal_false_len
    }
    pub fn prefix_string_len(&self) -> usize {
        self.prefix_string_len
    }
    pub fn dummy_comma(&self) -> &str {
        &self.dummy_comma
    }

    pub fn arr_start(&self, kind: BracketPaddingType) -> &str {
        &self.arr_start[kind as usize]
    }
    pub fn arr_end(&self, kind: BracketPaddingType) -> &str {
        &self.arr_end[kind as usize]
    }
    pub fn obj_start(&self, kind: BracketPaddingType) -> &str {
        &self.obj_start[kind as usize]
    }
    pub fn obj_end(&self, kind: BracketPaddingType) -> &str {
        &self.obj_end[kind as usize]
    }

    pub fn start(&self, elem_type: JsonItemType, bracket_type: BracketPaddingType) -> &str {
        if elem_type == JsonItemType::Array {
            self.arr_start(bracket_type)
        } else {
            self.obj_start(bracket_type)
        }
    }

    pub fn end(&self, elem_type: JsonItemType, bracket_type: BracketPaddingType) -> &str {
        if elem_type == JsonItemType::Array {
            self.arr_end(bracket_type)
        } else {
            self.obj_end(bracket_type)
        }
    }

    pub fn arr_start_len(&self, kind: BracketPaddingType) -> usize {
        self.arr_start_len[kind as usize]
    }
    pub fn arr_end_len(&self, kind: BracketPaddingType) -> usize {
        self.arr_end_len[kind as usize]
    }
    pub fn obj_start_len(&self, kind: BracketPaddingType) -> usize {
        self.obj_start_len[kind as usize]
    }
    pub fn obj_end_len(&self, kind: BracketPaddingType) -> usize {
        self.obj_end_len[kind as usize]
    }

    pub fn start_len(&self, elem_type: JsonItemType, bracket_type: BracketPaddingType) -> usize {
        if elem_type == JsonItemType::Array {
            self.arr_start_len(bracket_type)
        } else {
            self.obj_start_len(bracket_type)
        }
    }

    pub fn end_len(&self, elem_type: JsonItemType, bracket_type: BracketPaddingType) -> usize {
        if elem_type == JsonItemType::Array {
            self.arr_end_len(bracket_type)
        } else {
            self.obj_end_len(bracket_type)
        }
    }

    pub fn indent(&mut self, level: usize) -> String {
        if level >= self.indent_strings.len() {
            let base = self.indent_strings[1].clone();
            for i in self.indent_strings.len()..=level {
                let next = format!("{}{}", self.indent_strings[i - 1], base);
                self.indent_strings.push(next);
            }
        }
        self.indent_strings[level].clone()
    }
}
