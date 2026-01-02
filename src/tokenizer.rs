use crate::error::FracturedJsonError;
use crate::model::{InputPosition, JsonToken, TokenType};

const MAX_DOC_SIZE: usize = 2_000_000_000;

#[derive(Clone)]
pub struct ScannerState {
    original_text: String,
    chars: Vec<char>,
    byte_indices: Vec<usize>,
    pub current_position: InputPosition,
    pub token_position: InputPosition,
    pub non_whitespace_since_last_newline: bool,
}

impl ScannerState {
    pub fn new(original_text: &str) -> Self {
        let mut chars: Vec<char> = Vec::new();
        let mut byte_indices: Vec<usize> = Vec::new();
        for (idx, ch) in original_text.char_indices() {
            byte_indices.push(idx);
            chars.push(ch);
        }
        byte_indices.push(original_text.len());

        Self {
            original_text: original_text.to_string(),
            chars,
            byte_indices,
            current_position: InputPosition { index: 0, row: 0, column: 0 },
            token_position: InputPosition { index: 0, row: 0, column: 0 },
            non_whitespace_since_last_newline: false,
        }
    }

    pub fn advance(&mut self, is_whitespace: bool) {
        if self.current_position.index >= MAX_DOC_SIZE {
            panic!("Maximum document length exceeded");
        }
        self.current_position.index += 1;
        self.current_position.column += 1;
        if !is_whitespace {
            self.non_whitespace_since_last_newline = true;
        }
    }

    pub fn new_line(&mut self) {
        if self.current_position.index >= MAX_DOC_SIZE {
            panic!("Maximum document length exceeded");
        }
        self.current_position.index += 1;
        self.current_position.row += 1;
        self.current_position.column = 0;
        self.non_whitespace_since_last_newline = false;
    }

    pub fn set_token_start(&mut self) {
        self.token_position = self.current_position;
    }

    pub fn make_token_from_buffer(&self, token_type: TokenType, trim_end: bool) -> JsonToken {
        let start = self.byte_indices[self.token_position.index];
        let end = self.byte_indices[self.current_position.index];
        let mut substring = self.original_text[start..end].to_string();
        if trim_end {
            substring = substring.trim_end().to_string();
        }
        JsonToken {
            token_type,
            text: substring,
            input_position: self.token_position,
        }
    }

    pub fn make_token(&self, token_type: TokenType, text: &str) -> JsonToken {
        JsonToken {
            token_type,
            text: text.to_string(),
            input_position: self.token_position,
        }
    }

    pub fn current(&self) -> Option<char> {
        if self.at_end() {
            None
        } else {
            Some(self.chars[self.current_position.index])
        }
    }

    pub fn at_end(&self) -> bool {
        self.current_position.index >= self.chars.len()
    }

    pub fn error(&self, message: &str) -> FracturedJsonError {
        FracturedJsonError::new(message, Some(self.current_position))
    }
}

pub struct TokenGenerator {
    state: ScannerState,
}

impl TokenGenerator {
    pub fn new(input_json: &str) -> Self {
        Self { state: ScannerState::new(input_json) }
    }
}

impl Iterator for TokenGenerator {
    type Item = Result<JsonToken, FracturedJsonError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.state.at_end() {
                return None;
            }

            let ch = self.state.current()?;
            match ch {
                ' ' | '\t' | '\r' => {
                    self.state.advance(true);
                }
                '\n' => {
                    if !self.state.non_whitespace_since_last_newline {
                        return Some(Ok(self.state.make_token(TokenType::BlankLine, "\n")));
                    }
                    self.state.new_line();
                    self.state.set_token_start();
                }
                '{' => return Some(process_single_char(&mut self.state, "{", TokenType::BeginObject)),
                '}' => return Some(process_single_char(&mut self.state, "}", TokenType::EndObject)),
                '[' => return Some(process_single_char(&mut self.state, "[", TokenType::BeginArray)),
                ']' => return Some(process_single_char(&mut self.state, "]", TokenType::EndArray)),
                ':' => return Some(process_single_char(&mut self.state, ":", TokenType::Colon)),
                ',' => return Some(process_single_char(&mut self.state, ",", TokenType::Comma)),
                't' => return Some(process_keyword(&mut self.state, "true", TokenType::True)),
                'f' => return Some(process_keyword(&mut self.state, "false", TokenType::False)),
                'n' => return Some(process_keyword(&mut self.state, "null", TokenType::Null)),
                '/' => return Some(process_comment(&mut self.state)),
                '"' => return Some(process_string(&mut self.state)),
                '-' => return Some(process_number(&mut self.state)),
                _ => {
                    if !is_digit(ch) {
                        return Some(Err(self.state.error("Unexpected character")));
                    }
                    return Some(process_number(&mut self.state));
                }
            }
        }
    }
}

fn process_single_char(
    state: &mut ScannerState,
    symbol: &str,
    token_type: TokenType,
) -> Result<JsonToken, FracturedJsonError> {
    state.set_token_start();
    let token = state.make_token(token_type, symbol);
    state.advance(false);
    Ok(token)
}

fn process_keyword(
    state: &mut ScannerState,
    keyword: &str,
    token_type: TokenType,
) -> Result<JsonToken, FracturedJsonError> {
    state.set_token_start();
    let mut chars = keyword.chars();
    chars.next();
    for expected in chars {
        if state.at_end() {
            return Err(state.error("Unexpected end of input while processing keyword"));
        }
        state.advance(false);
        let current = state.current().unwrap();
        if current != expected {
            return Err(state.error("Unexpected keyword"));
        }
    }

    let token = state.make_token(token_type, keyword);
    state.advance(false);
    Ok(token)
}

fn process_comment(state: &mut ScannerState) -> Result<JsonToken, FracturedJsonError> {
    state.set_token_start();

    if state.at_end() {
        return Err(state.error("Unexpected end of input while processing comment"));
    }

    state.advance(false);
    let mut is_block_comment = false;
    match state.current() {
        Some('*') => is_block_comment = true,
        Some('/') => {}
        _ => return Err(state.error("Bad character for start of comment")),
    }

    state.advance(false);
    let mut last_char_was_asterisk = false;
    loop {
        if state.at_end() {
            if is_block_comment {
                return Err(state.error("Unexpected end of input while processing comment"));
            }
            return Ok(state.make_token_from_buffer(TokenType::LineComment, true));
        }

        let ch = state.current().unwrap();
        if ch == '\n' {
            state.new_line();
            if !is_block_comment {
                return Ok(state.make_token_from_buffer(TokenType::LineComment, true));
            }
            continue;
        }

        state.advance(false);
        if ch == '/' && last_char_was_asterisk {
            return Ok(state.make_token_from_buffer(TokenType::BlockComment, false));
        }
        last_char_was_asterisk = ch == '*';
    }
}

fn process_string(state: &mut ScannerState) -> Result<JsonToken, FracturedJsonError> {
    state.set_token_start();
    state.advance(false);

    let mut last_char_began_escape = false;
    let mut expected_hex_count = 0usize;
    loop {
        if state.at_end() {
            return Err(state.error("Unexpected end of input while processing string"));
        }

        let ch = state.current().unwrap();

        if expected_hex_count > 0 {
            if !is_hex(ch) {
                return Err(state.error("Bad unicode escape in string"));
            }
            expected_hex_count -= 1;
            state.advance(false);
            continue;
        }

        if last_char_began_escape {
            if !is_legal_after_backslash(ch) {
                return Err(state.error("Bad escaped character in string"));
            }
            if ch == 'u' {
                expected_hex_count = 4;
            }
            last_char_began_escape = false;
            state.advance(false);
            continue;
        }

        if is_control(ch) {
            return Err(state.error("Control characters are not allowed in strings"));
        }

        state.advance(false);
        if ch == '"' {
            return Ok(state.make_token_from_buffer(TokenType::String, false));
        }
        if ch == '\\' {
            last_char_began_escape = true;
        }
    }
}

fn process_number(state: &mut ScannerState) -> Result<JsonToken, FracturedJsonError> {
    state.set_token_start();
    let mut phase = NumberPhase::Beginning;
    loop {
        let ch = state.current().unwrap();
        let mut handling = CharHandling::ValidAndConsumed;

        match phase {
            NumberPhase::Beginning => {
                if ch == '-' {
                    phase = NumberPhase::PastLeadingSign;
                } else if ch == '0' {
                    phase = NumberPhase::PastWhole;
                } else if is_digit(ch) {
                    phase = NumberPhase::PastFirstDigitOfWhole;
                } else {
                    handling = CharHandling::InvalidatesToken;
                }
            }
            NumberPhase::PastLeadingSign => {
                if !is_digit(ch) {
                    handling = CharHandling::InvalidatesToken;
                } else if ch == '0' {
                    phase = NumberPhase::PastWhole;
                } else {
                    phase = NumberPhase::PastFirstDigitOfWhole;
                }
            }
            NumberPhase::PastFirstDigitOfWhole => {
                if ch == '.' {
                    phase = NumberPhase::PastDecimalPoint;
                } else if ch == 'e' || ch == 'E' {
                    phase = NumberPhase::PastE;
                } else if !is_digit(ch) {
                    handling = CharHandling::StartOfNewToken;
                }
            }
            NumberPhase::PastWhole => {
                if ch == '.' {
                    phase = NumberPhase::PastDecimalPoint;
                } else if ch == 'e' || ch == 'E' {
                    phase = NumberPhase::PastE;
                } else {
                    handling = CharHandling::StartOfNewToken;
                }
            }
            NumberPhase::PastDecimalPoint => {
                if is_digit(ch) {
                    phase = NumberPhase::PastFirstDigitOfFractional;
                } else {
                    handling = CharHandling::InvalidatesToken;
                }
            }
            NumberPhase::PastFirstDigitOfFractional => {
                if ch == 'e' || ch == 'E' {
                    phase = NumberPhase::PastE;
                } else if !is_digit(ch) {
                    handling = CharHandling::StartOfNewToken;
                }
            }
            NumberPhase::PastE => {
                if ch == '+' || ch == '-' {
                    phase = NumberPhase::PastExpSign;
                } else if is_digit(ch) {
                    phase = NumberPhase::PastFirstDigitOfExponent;
                } else {
                    handling = CharHandling::InvalidatesToken;
                }
            }
            NumberPhase::PastExpSign => {
                if is_digit(ch) {
                    phase = NumberPhase::PastFirstDigitOfExponent;
                } else {
                    handling = CharHandling::InvalidatesToken;
                }
            }
            NumberPhase::PastFirstDigitOfExponent => {
                if !is_digit(ch) {
                    handling = CharHandling::StartOfNewToken;
                }
            }
        }

        if handling == CharHandling::InvalidatesToken {
            return Err(state.error("Bad character while processing number"));
        }

        if handling == CharHandling::StartOfNewToken {
            return Ok(state.make_token_from_buffer(TokenType::Number, false));
        }

        if !state.at_end() {
            state.advance(false);
            continue;
        }

        return match phase {
            NumberPhase::PastFirstDigitOfWhole
            | NumberPhase::PastWhole
            | NumberPhase::PastFirstDigitOfFractional
            | NumberPhase::PastFirstDigitOfExponent => Ok(state.make_token_from_buffer(TokenType::Number, false)),
            _ => Err(state.error("Unexpected end of input while processing number")),
        };
    }
}

fn is_digit(ch: char) -> bool {
    ch >= '0' && ch <= '9'
}

fn is_hex(ch: char) -> bool {
    (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')
}

fn is_legal_after_backslash(ch: char) -> bool {
    matches!(ch, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u')
}

fn is_control(ch: char) -> bool {
    let code = ch as u32;
    (code <= 0x1F) || (code == 0x7F) || (code >= 0x80 && code <= 0x9F)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumberPhase {
    Beginning,
    PastLeadingSign,
    PastFirstDigitOfWhole,
    PastWhole,
    PastDecimalPoint,
    PastFirstDigitOfFractional,
    PastE,
    PastExpSign,
    PastFirstDigitOfExponent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharHandling {
    InvalidatesToken,
    ValidAndConsumed,
    StartOfNewToken,
}
