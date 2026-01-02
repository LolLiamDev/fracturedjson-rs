use crate::error::FracturedJsonError;
use crate::model::{InputPosition, JsonItem, JsonItemType, JsonToken, TokenType};
use crate::options::{CommentPolicy, FracturedJsonOptions};
use crate::tokenizer::TokenGenerator;

pub struct TokenEnumerator<I>
where
    I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
{
    generator: I,
    current: Option<JsonToken>,
}

impl<I> TokenEnumerator<I>
where
    I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
{
    pub fn new(generator: I) -> Self {
        Self { generator, current: None }
    }

    pub fn current(&self) -> Result<&JsonToken, FracturedJsonError> {
        self.current.as_ref().ok_or_else(|| FracturedJsonError::simple("Illegal enumerator usage"))
    }

    pub fn move_next(&mut self) -> Result<bool, FracturedJsonError> {
        match self.generator.next() {
            None => {
                self.current = None;
                Ok(false)
            }
            Some(Ok(token)) => {
                self.current = Some(token);
                Ok(true)
            }
            Some(Err(err)) => Err(err),
        }
    }
}

pub struct Parser {
    pub options: FracturedJsonOptions,
}

impl Parser {
    pub fn new(options: FracturedJsonOptions) -> Self {
        Self { options }
    }

    pub fn parse_top_level(
        &self,
        input_json: &str,
        stop_after_first_elem: bool,
    ) -> Result<Vec<JsonItem>, FracturedJsonError> {
        let token_stream = TokenGenerator::new(input_json);
        let mut enumerator = TokenEnumerator::new(token_stream);
        self.parse_top_level_from_enum(&mut enumerator, stop_after_first_elem)
    }

    fn parse_top_level_from_enum<I>(
        &self,
        enumerator: &mut TokenEnumerator<I>,
        stop_after_first_elem: bool,
    ) -> Result<Vec<JsonItem>, FracturedJsonError>
    where
        I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
    {
        let mut top_level_items: Vec<JsonItem> = Vec::new();
        let mut top_level_elem_seen = false;

        loop {
            if !enumerator.move_next()? {
                return Ok(top_level_items);
            }

            let item = self.parse_item(enumerator)?;
            let is_comment = matches!(item.item_type, JsonItemType::BlockComment | JsonItemType::LineComment);
            let is_blank = item.item_type == JsonItemType::BlankLine;

            if is_blank {
                if self.options.preserve_blank_lines {
                    top_level_items.push(item);
                }
            } else if is_comment {
                match self.options.comment_policy {
                    CommentPolicy::TreatAsError => {
                        return Err(FracturedJsonError::new(
                            "Comments not allowed with current options",
                            Some(item.input_position),
                        ));
                    }
                    CommentPolicy::Preserve => top_level_items.push(item),
                    CommentPolicy::Remove => {}
                }
            } else {
                if stop_after_first_elem && top_level_elem_seen {
                    return Err(FracturedJsonError::new(
                        "Unexpected start of second top level element",
                        Some(item.input_position),
                    ));
                }
                top_level_items.push(item);
                top_level_elem_seen = true;
            }
        }
    }

    fn parse_item<I>(&self, enumerator: &mut TokenEnumerator<I>) -> Result<JsonItem, FracturedJsonError>
    where
        I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
    {
        let current = enumerator.current()?.clone();
        match current.token_type {
            TokenType::BeginArray => self.parse_array(enumerator),
            TokenType::BeginObject => self.parse_object(enumerator),
            _ => Ok(self.parse_simple(&current)),
        }
    }

    fn parse_simple(&self, token: &JsonToken) -> JsonItem {
        let mut item = JsonItem::default();
        item.item_type = Self::item_type_from_token_type(token);
        item.value = token.text.clone();
        item.input_position = token.input_position;
        item.complexity = 0;
        item
    }

    fn parse_array<I>(&self, enumerator: &mut TokenEnumerator<I>) -> Result<JsonItem, FracturedJsonError>
    where
        I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
    {
        if enumerator.current()?.token_type != TokenType::BeginArray {
            return Err(FracturedJsonError::new(
                "Parser logic error",
                Some(enumerator.current()?.input_position),
            ));
        }

        let starting_input_position = enumerator.current()?.input_position;

        let mut elem_needing_post_comment_idx: Option<usize> = None;
        let mut elem_needing_post_end_row: isize = -1;

        let mut unplaced_comment: Option<JsonItem> = None;
        let mut child_list: Vec<JsonItem> = Vec::new();
        let mut comma_status = CommaStatus::EmptyCollection;
        let mut end_of_array_found = false;
        let mut this_array_complexity = 0usize;

        while !end_of_array_found {
            let token = Self::get_next_token_or_throw(enumerator, starting_input_position)?;

            let unplaced_needs_home = unplaced_comment.as_ref().map(|comment| {
                comment.input_position.row != token.input_position.row || token.token_type == TokenType::EndArray
            }).unwrap_or(false);

            if unplaced_needs_home {
                if let Some(idx) = elem_needing_post_comment_idx {
                    if let Some(elem) = child_list.get_mut(idx) {
                        elem.postfix_comment = unplaced_comment.as_ref().unwrap().value.clone();
                        elem.is_post_comment_line_style = unplaced_comment.as_ref().unwrap().item_type == JsonItemType::LineComment;
                    }
                } else {
                    child_list.push(unplaced_comment.as_ref().unwrap().clone());
                }
                unplaced_comment = None;
            }

            if elem_needing_post_comment_idx.is_some()
                && elem_needing_post_end_row != token.input_position.row as isize
            {
                elem_needing_post_comment_idx = None;
            }

            match token.token_type {
                TokenType::EndArray => {
                    if comma_status == CommaStatus::CommaSeen && !self.options.allow_trailing_commas {
                        return Err(FracturedJsonError::new(
                            "Array may not end with a comma with current options",
                            Some(token.input_position),
                        ));
                    }
                    end_of_array_found = true;
                }
                TokenType::Comma => {
                    if comma_status != CommaStatus::ElementSeen {
                        return Err(FracturedJsonError::new(
                            "Unexpected comma in array",
                            Some(token.input_position),
                        ));
                    }
                    comma_status = CommaStatus::CommaSeen;
                }
                TokenType::BlankLine => {
                    if self.options.preserve_blank_lines {
                        child_list.push(self.parse_simple(&token));
                    }
                }
                TokenType::BlockComment => {
                    if self.options.comment_policy == CommentPolicy::Remove {
                        continue;
                    }
                    if self.options.comment_policy == CommentPolicy::TreatAsError {
                        return Err(FracturedJsonError::new(
                            "Comments not allowed with current options",
                            Some(token.input_position),
                        ));
                    }

                    if unplaced_comment.is_some() {
                        child_list.push(unplaced_comment.take().unwrap());
                    }

                    let comment_item = self.parse_simple(&token);
                    if Self::is_multiline_comment(&comment_item) {
                        child_list.push(comment_item);
                        continue;
                    }

                    if let Some(idx) = elem_needing_post_comment_idx {
                        if comma_status == CommaStatus::ElementSeen {
                            if let Some(elem) = child_list.get_mut(idx) {
                                elem.postfix_comment = comment_item.value.clone();
                                elem.is_post_comment_line_style = false;
                            }
                            elem_needing_post_comment_idx = None;
                            continue;
                        }
                    }

                    unplaced_comment = Some(comment_item);
                }
                TokenType::LineComment => {
                    if self.options.comment_policy == CommentPolicy::Remove {
                        continue;
                    }
                    if self.options.comment_policy == CommentPolicy::TreatAsError {
                        return Err(FracturedJsonError::new(
                            "Comments not allowed with current options",
                            Some(token.input_position),
                        ));
                    }

                    if unplaced_comment.is_some() {
                        child_list.push(unplaced_comment.take().unwrap());
                        child_list.push(self.parse_simple(&token));
                        continue;
                    }

                    if let Some(idx) = elem_needing_post_comment_idx {
                        if let Some(elem) = child_list.get_mut(idx) {
                            elem.postfix_comment = token.text.clone();
                            elem.is_post_comment_line_style = true;
                        }
                        elem_needing_post_comment_idx = None;
                        continue;
                    }

                    child_list.push(self.parse_simple(&token));
                }
                TokenType::False
                | TokenType::True
                | TokenType::Null
                | TokenType::String
                | TokenType::Number
                | TokenType::BeginArray
                | TokenType::BeginObject => {
                    if comma_status == CommaStatus::ElementSeen {
                        return Err(FracturedJsonError::new(
                            "Comma missing while processing array",
                            Some(token.input_position),
                        ));
                    }

                    let mut element = self.parse_item(enumerator)?;
                    comma_status = CommaStatus::ElementSeen;
                    this_array_complexity = this_array_complexity.max(element.complexity + 1);

                    if let Some(unplaced) = unplaced_comment.take() {
                        element.prefix_comment = unplaced.value;
                    }

                    child_list.push(element);
                    elem_needing_post_comment_idx = Some(child_list.len() - 1);
                    elem_needing_post_end_row = enumerator.current()?.input_position.row as isize;
                }
                _ => {
                    return Err(FracturedJsonError::new(
                        "Unexpected token in array",
                        Some(token.input_position),
                    ));
                }
            }
        }

        let mut array_item = JsonItem::default();
        array_item.item_type = JsonItemType::Array;
        array_item.input_position = starting_input_position;
        array_item.complexity = this_array_complexity;
        array_item.children = child_list;
        Ok(array_item)
    }

    fn parse_object<I>(&self, enumerator: &mut TokenEnumerator<I>) -> Result<JsonItem, FracturedJsonError>
    where
        I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
    {
        if enumerator.current()?.token_type != TokenType::BeginObject {
            return Err(FracturedJsonError::new("Parser logic error", Some(enumerator.current()?.input_position)));
        }

        let starting_input_position = enumerator.current()?.input_position;
        let mut child_list: Vec<JsonItem> = Vec::new();

        let mut property_name: Option<JsonToken> = None;
        let mut property_value: Option<JsonItem> = None;
        let mut line_prop_value_ends: isize = -1;
        let mut before_prop_comments: Vec<JsonItem> = Vec::new();
        let mut mid_prop_comments: Vec<JsonToken> = Vec::new();
        let mut after_prop_comment: Option<JsonItem> = None;
        let mut after_prop_comment_was_after_comma = false;

        let mut phase = ObjectPhase::BeforePropName;
        let mut this_obj_complexity = 0usize;
        let mut end_of_object = false;
        while !end_of_object {
            let token = Self::get_next_token_or_throw(enumerator, starting_input_position)?;

            let is_new_line = line_prop_value_ends != token.input_position.row as isize;
            let is_end_of_object = token.token_type == TokenType::EndObject;
            let starting_next_prop_name = token.token_type == TokenType::String && phase == ObjectPhase::AfterComma;
            let is_excess_post_comment = after_prop_comment.is_some()
                && matches!(token.token_type, TokenType::BlockComment | TokenType::LineComment);

            let need_to_flush = property_name.is_some()
                && property_value.is_some()
                && (is_new_line || is_end_of_object || starting_next_prop_name || is_excess_post_comment);

            if need_to_flush {
                let mut comment_to_hold_for_next_elem: Option<JsonItem> = None;
                if starting_next_prop_name && after_prop_comment_was_after_comma && !is_new_line {
                    comment_to_hold_for_next_elem = after_prop_comment.take();
                }

                Self::attach_object_value_pieces(
                    &mut child_list,
                    property_name.as_ref().unwrap(),
                    property_value.as_ref().unwrap(),
                    line_prop_value_ends,
                    &mut before_prop_comments,
                    &mut mid_prop_comments,
                    after_prop_comment.take(),
                );
                this_obj_complexity = this_obj_complexity.max(property_value.as_ref().unwrap().complexity + 1);
                property_name = None;
                property_value = None;
                before_prop_comments.clear();
                mid_prop_comments.clear();
                after_prop_comment = None;

                if let Some(comment) = comment_to_hold_for_next_elem {
                    before_prop_comments.push(comment);
                }
            }

            match token.token_type {
                TokenType::BlankLine => {
                    if !self.options.preserve_blank_lines {
                        continue;
                    }
                    if matches!(phase, ObjectPhase::AfterPropName | ObjectPhase::AfterColon) {
                        continue;
                    }
                    child_list.extend(before_prop_comments.drain(..));
                    child_list.push(self.parse_simple(&token));
                }
                TokenType::BlockComment | TokenType::LineComment => {
                    if self.options.comment_policy == CommentPolicy::Remove {
                        continue;
                    }
                    if self.options.comment_policy == CommentPolicy::TreatAsError {
                        return Err(FracturedJsonError::new(
                            "Comments not allowed with current options",
                            Some(token.input_position),
                        ));
                    }
                    if matches!(phase, ObjectPhase::BeforePropName) || property_name.is_none() {
                        before_prop_comments.push(self.parse_simple(&token));
                    } else if matches!(phase, ObjectPhase::AfterPropName | ObjectPhase::AfterColon) {
                        mid_prop_comments.push(token);
                    } else {
                        after_prop_comment = Some(self.parse_simple(&token));
                        after_prop_comment_was_after_comma = matches!(phase, ObjectPhase::AfterComma);
                    }
                }
                TokenType::EndObject => {
                    if matches!(phase, ObjectPhase::AfterPropName | ObjectPhase::AfterColon) {
                        return Err(FracturedJsonError::new("Unexpected end of object", Some(token.input_position)));
                    }
                    end_of_object = true;
                }
                TokenType::String => {
                    if matches!(phase, ObjectPhase::BeforePropName | ObjectPhase::AfterComma) {
                        property_name = Some(token);
                        phase = ObjectPhase::AfterPropName;
                    } else if matches!(phase, ObjectPhase::AfterColon) {
                        property_value = Some(self.parse_item(enumerator)?);
                        line_prop_value_ends = enumerator.current()?.input_position.row as isize;
                        phase = ObjectPhase::AfterPropValue;
                    } else {
                        return Err(FracturedJsonError::new(
                            "Unexpected string found while processing object",
                            Some(token.input_position),
                        ));
                    }
                }
                TokenType::False
                | TokenType::True
                | TokenType::Null
                | TokenType::Number
                | TokenType::BeginArray
                | TokenType::BeginObject => {
                    if !matches!(phase, ObjectPhase::AfterColon) {
                        return Err(FracturedJsonError::new(
                            "Unexpected element while processing object",
                            Some(token.input_position),
                        ));
                    }
                    property_value = Some(self.parse_item(enumerator)?);
                    line_prop_value_ends = enumerator.current()?.input_position.row as isize;
                    phase = ObjectPhase::AfterPropValue;
                }
                TokenType::Colon => {
                    if !matches!(phase, ObjectPhase::AfterPropName) {
                        return Err(FracturedJsonError::new(
                            "Unexpected colon while processing object",
                            Some(token.input_position),
                        ));
                    }
                    phase = ObjectPhase::AfterColon;
                }
                TokenType::Comma => {
                    if !matches!(phase, ObjectPhase::AfterPropValue) {
                        return Err(FracturedJsonError::new(
                            "Unexpected comma while processing object",
                            Some(token.input_position),
                        ));
                    }
                    phase = ObjectPhase::AfterComma;
                }
                _ => {
                    return Err(FracturedJsonError::new(
                        "Unexpected token while processing object",
                        Some(token.input_position),
                    ));
                }
            }
        }

        if !self.options.allow_trailing_commas && matches!(phase, ObjectPhase::AfterComma) {
            return Err(FracturedJsonError::new(
                "Object may not end with comma with current options",
                Some(enumerator.current()?.input_position),
            ));
        }

        let mut obj_item = JsonItem::default();
        obj_item.item_type = JsonItemType::Object;
        obj_item.input_position = starting_input_position;
        obj_item.complexity = this_obj_complexity;
        obj_item.children = child_list;
        Ok(obj_item)
    }

    fn item_type_from_token_type(token: &JsonToken) -> JsonItemType {
        match token.token_type {
            TokenType::False => JsonItemType::False,
            TokenType::True => JsonItemType::True,
            TokenType::Null => JsonItemType::Null,
            TokenType::Number => JsonItemType::Number,
            TokenType::String => JsonItemType::String,
            TokenType::BlankLine => JsonItemType::BlankLine,
            TokenType::BlockComment => JsonItemType::BlockComment,
            TokenType::LineComment => JsonItemType::LineComment,
            _ => panic!("Unexpected Token"),
        }
    }

    fn get_next_token_or_throw<I>(
        enumerator: &mut TokenEnumerator<I>,
        start_position: InputPosition,
    ) -> Result<JsonToken, FracturedJsonError>
    where
        I: Iterator<Item = Result<JsonToken, FracturedJsonError>>,
    {
        if !enumerator.move_next()? {
            return Err(FracturedJsonError::new(
                "Unexpected end of input while processing array or object starting",
                Some(start_position),
            ));
        }
        Ok(enumerator.current()?.clone())
    }

    fn is_multiline_comment(item: &JsonItem) -> bool {
        item.item_type == JsonItemType::BlockComment && item.value.contains('\n')
    }

    fn attach_object_value_pieces(
        obj_item_list: &mut Vec<JsonItem>,
        name: &JsonToken,
        element: &JsonItem,
        value_ending_line: isize,
        before_comments: &mut Vec<JsonItem>,
        mid_comments: &mut Vec<JsonToken>,
        after_comment: Option<JsonItem>,
    ) {
        let mut element = element.clone();
        element.name = name.text.clone();

        if !mid_comments.is_empty() {
            let mut combined = String::new();
            for (i, comment) in mid_comments.iter().enumerate() {
                combined.push_str(&comment.text);
                if i < mid_comments.len() - 1 || comment.token_type == TokenType::LineComment {
                    combined.push('\n');
                }
            }
            element.middle_comment = combined.clone();
            element.middle_comment_has_new_line = combined.contains('\n');
        }

        if !before_comments.is_empty() {
            let last = before_comments.pop().unwrap();
            if last.item_type == JsonItemType::BlockComment
                && last.input_position.row == element.input_position.row
            {
                element.prefix_comment = last.value;
                obj_item_list.extend(before_comments.drain(..));
            } else {
                obj_item_list.extend(before_comments.drain(..));
                obj_item_list.push(last);
            }
        }

        obj_item_list.push(element.clone());

        if let Some(after) = after_comment {
            if !Self::is_multiline_comment(&after)
                && after.input_position.row as isize == value_ending_line
            {
                let mut updated = element.clone();
                updated.postfix_comment = after.value;
                updated.is_post_comment_line_style = after.item_type == JsonItemType::LineComment;
                obj_item_list.pop();
                obj_item_list.push(updated);
            } else {
                obj_item_list.push(after);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommaStatus {
    EmptyCollection,
    ElementSeen,
    CommaSeen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectPhase {
    BeforePropName,
    AfterPropName,
    AfterColon,
    AfterPropValue,
    AfterComma,
}
