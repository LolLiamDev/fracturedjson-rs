use crate::buffer::PaddedFormattingTokens;
use crate::buffer::StringJoinBuffer;
use crate::model::{BracketPaddingType, JsonItem, JsonItemType, TableColumnType};
use crate::options::NumberListAlignment;

#[derive(Debug, Clone)]
pub struct TableTemplate {
    pub location_in_parent: Option<String>,
    pub column_type: TableColumnType,
    pub row_count: usize,
    pub name_length: usize,
    pub name_minimum: usize,
    pub max_value_length: usize,
    pub max_atomic_value_length: usize,
    pub prefix_comment_length: usize,
    pub middle_comment_length: usize,
    pub any_middle_comment_has_newline: bool,
    pub postfix_comment_length: usize,
    pub is_any_post_comment_line_style: bool,
    pub pad_type: BracketPaddingType,
    pub requires_multiple_lines: bool,
    pub composite_value_length: usize,
    pub total_length: usize,
    pub shorter_than_null_adjustment: usize,
    pub contains_null: bool,
    pub children: Vec<TableTemplate>,
    pads: PaddedFormattingTokens,
    number_list_alignment: NumberListAlignment,
    max_dig_before_dec: usize,
    max_dig_after_dec: usize,
}

impl TableTemplate {
    pub fn new(pads: PaddedFormattingTokens, number_list_alignment: NumberListAlignment) -> Self {
        Self {
            location_in_parent: None,
            column_type: TableColumnType::Unknown,
            row_count: 0,
            name_length: 0,
            name_minimum: usize::MAX,
            max_value_length: 0,
            max_atomic_value_length: 0,
            prefix_comment_length: 0,
            middle_comment_length: 0,
            any_middle_comment_has_newline: false,
            postfix_comment_length: 0,
            is_any_post_comment_line_style: false,
            pad_type: BracketPaddingType::Simple,
            requires_multiple_lines: false,
            composite_value_length: 0,
            total_length: 0,
            shorter_than_null_adjustment: 0,
            contains_null: false,
            children: Vec::new(),
            pads,
            number_list_alignment,
            max_dig_before_dec: 0,
            max_dig_after_dec: 0,
        }
    }

    pub fn measure_table_root(&mut self, table_root: &JsonItem, recursive: bool) {
        for child in &table_root.children {
            self.measure_row_segment(child, recursive);
        }
        self.prune_and_recompute(usize::MAX);
    }

    pub fn try_to_fit(&mut self, maximum_length: usize) -> bool {
        let mut complexity = self.get_template_complexity();
        loop {
            if self.total_length <= maximum_length {
                return true;
            }
            if complexity == 0 {
                return false;
            }
            complexity -= 1;
            self.prune_and_recompute(complexity);
        }
    }

    pub fn format_number(
        &self,
        buffer: &mut StringJoinBuffer,
        item: &JsonItem,
        comma_before_pad_type: &str,
    ) {
        match self.number_list_alignment {
            NumberListAlignment::Left => {
                buffer
                    .add(&item.value)
                    .add(comma_before_pad_type)
                    .spaces(self.max_value_length - item.value_length);
                return;
            }
            NumberListAlignment::Right => {
                buffer
                    .spaces(self.max_value_length - item.value_length)
                    .add(&item.value)
                    .add(comma_before_pad_type);
                return;
            }
            _ => {}
        }

        if item.item_type == JsonItemType::Null {
            buffer
                .spaces(self.max_dig_before_dec.saturating_sub(item.value_length))
                .add(&item.value)
                .add(comma_before_pad_type)
                .spaces(self.composite_value_length - self.max_dig_before_dec);
            return;
        }

        if self.number_list_alignment == NumberListAlignment::Normalize {
            let parsed_val: f64 = item.value.parse().unwrap_or(f64::NAN);
            let reformatted = format!("{:.*}", self.max_dig_after_dec, parsed_val);
            buffer
                .spaces(self.composite_value_length - reformatted.len())
                .add(&reformatted)
                .add(comma_before_pad_type);
            return;
        }

        let index_of_dot = dot_or_e_index(&item.value);
        let (left_pad, right_pad) = if let Some(dot) = index_of_dot {
            let left_pad = self.max_dig_before_dec.saturating_sub(dot);
            let right_pad = self
                .composite_value_length
                .saturating_sub(left_pad + item.value_length);
            (left_pad, right_pad)
        } else {
            let left_pad = self.max_dig_before_dec.saturating_sub(item.value_length);
            let right_pad = self
                .composite_value_length
                .saturating_sub(self.max_dig_before_dec);
            (left_pad, right_pad)
        };

        buffer
            .spaces(left_pad)
            .add(&item.value)
            .add(comma_before_pad_type)
            .spaces(right_pad);
    }

    pub fn atomic_item_size(&self) -> usize {
        self.name_length
            + self.pads.colon_len()
            + self.middle_comment_length
            + if self.middle_comment_length > 0 {
                self.pads.comment_len()
            } else {
                0
            }
            + self.max_atomic_value_length
            + self.postfix_comment_length
            + if self.postfix_comment_length > 0 {
                self.pads.comment_len()
            } else {
                0
            }
            + self.pads.comma_len()
    }

    fn measure_row_segment(&mut self, row_segment: &JsonItem, recursive: bool) {
        if matches!(
            row_segment.item_type,
            JsonItemType::BlankLine | JsonItemType::BlockComment | JsonItemType::LineComment
        ) {
            return;
        }

        let row_table_type = match row_segment.item_type {
            JsonItemType::Null => TableColumnType::Unknown,
            JsonItemType::Number => TableColumnType::Number,
            JsonItemType::Array => TableColumnType::Array,
            JsonItemType::Object => TableColumnType::Object,
            _ => TableColumnType::Simple,
        };

        if self.column_type == TableColumnType::Unknown {
            self.column_type = row_table_type;
        } else if row_table_type != TableColumnType::Unknown && self.column_type != row_table_type {
            self.column_type = TableColumnType::Mixed;
        }

        if row_segment.item_type == JsonItemType::Null {
            self.max_dig_before_dec = self.max_dig_before_dec.max(self.pads.literal_null_len());
            self.contains_null = true;
        }

        if row_segment.requires_multiple_lines {
            self.requires_multiple_lines = true;
            self.column_type = TableColumnType::Mixed;
        }

        self.row_count += 1;
        self.name_length = self.name_length.max(row_segment.name_length);
        self.name_minimum = self.name_minimum.min(row_segment.name_length);
        self.max_value_length = self.max_value_length.max(row_segment.value_length);
        self.middle_comment_length = self
            .middle_comment_length
            .max(row_segment.middle_comment_length);
        self.prefix_comment_length = self
            .prefix_comment_length
            .max(row_segment.prefix_comment_length);
        self.postfix_comment_length = self
            .postfix_comment_length
            .max(row_segment.postfix_comment_length);
        self.is_any_post_comment_line_style |= row_segment.is_post_comment_line_style;
        self.any_middle_comment_has_newline |= row_segment.middle_comment_has_new_line;

        if !matches!(
            row_segment.item_type,
            JsonItemType::Array | JsonItemType::Object
        ) {
            self.max_atomic_value_length =
                self.max_atomic_value_length.max(row_segment.value_length);
        }

        if row_segment.complexity >= 2 {
            self.pad_type = BracketPaddingType::Complex;
        }

        if self.requires_multiple_lines || row_segment.item_type == JsonItemType::Null {
            return;
        }

        if self.column_type == TableColumnType::Array && recursive {
            for (i, child) in row_segment.children.iter().enumerate() {
                if self.children.len() <= i {
                    self.children.push(TableTemplate::new(
                        self.pads.clone(),
                        self.number_list_alignment,
                    ));
                }
                self.children[i].measure_row_segment(child, true);
            }
        } else if self.column_type == TableColumnType::Object && recursive {
            if contains_duplicate_keys(&row_segment.children) {
                self.column_type = TableColumnType::Simple;
                return;
            }

            for row_child in &row_segment.children {
                let mut idx = None;
                for (i, child) in self.children.iter().enumerate() {
                    if child.location_in_parent.as_deref() == Some(&row_child.name) {
                        idx = Some(i);
                        break;
                    }
                }

                if let Some(index) = idx {
                    self.children[index].measure_row_segment(row_child, true);
                } else {
                    let mut sub_template =
                        TableTemplate::new(self.pads.clone(), self.number_list_alignment);
                    sub_template.location_in_parent = Some(row_child.name.clone());
                    sub_template.measure_row_segment(row_child, true);
                    self.children.push(sub_template);
                }
            }
        }

        let skip_decimal = self.column_type != TableColumnType::Number
            || matches!(
                self.number_list_alignment,
                NumberListAlignment::Left | NumberListAlignment::Right
            );
        if skip_decimal {
            return;
        }

        let mut normalized_str = row_segment.value.clone();
        if self.number_list_alignment == NumberListAlignment::Normalize {
            let parsed_val: f64 = normalized_str.parse().unwrap_or(f64::NAN);
            normalized_str = parsed_val.to_string();

            let can_normalize = parsed_val.is_finite()
                && normalized_str.len() <= 16
                && !normalized_str.contains('e')
                && (parsed_val != 0.0 || is_truly_zero(&row_segment.value));
            if !can_normalize {
                self.number_list_alignment = NumberListAlignment::Left;
                return;
            }
        }

        let index_of_dot = dot_or_e_index(&normalized_str);
        let before_dec = match index_of_dot {
            Some(idx) => idx,
            None => normalized_str.len(),
        };
        let after_dec = match index_of_dot {
            Some(idx) => normalized_str.len().saturating_sub(idx + 1),
            None => 0,
        };
        self.max_dig_before_dec = self.max_dig_before_dec.max(before_dec);
        self.max_dig_after_dec = self.max_dig_after_dec.max(after_dec);
    }

    fn prune_and_recompute(&mut self, max_allowed_complexity: usize) {
        let clear_children = max_allowed_complexity == 0
            || (!matches!(
                self.column_type,
                TableColumnType::Array | TableColumnType::Object
            ))
            || self.row_count < 2;
        if clear_children {
            self.children.clear();
        }

        for child in &mut self.children {
            child.prune_and_recompute(max_allowed_complexity.saturating_sub(1));
        }

        if self.column_type == TableColumnType::Number {
            self.composite_value_length = self.get_number_field_width();
        } else if !self.children.is_empty() {
            let total_child_len: usize = self.children.iter().map(|ch| ch.total_length).sum();
            self.composite_value_length = total_child_len
                + self
                    .pads
                    .comma_len()
                    .saturating_mul(self.children.len().saturating_sub(1))
                + self.pads.arr_start_len(self.pad_type)
                + self.pads.arr_end_len(self.pad_type);
            if self.contains_null && self.composite_value_length < self.pads.literal_null_len() {
                self.shorter_than_null_adjustment =
                    self.pads.literal_null_len() - self.composite_value_length;
                self.composite_value_length = self.pads.literal_null_len();
            }
        } else {
            self.composite_value_length = self.max_value_length;
        }

        self.total_length = (if self.prefix_comment_length > 0 {
            self.prefix_comment_length + self.pads.comment_len()
        } else {
            0
        }) + (if self.name_length > 0 {
            self.name_length + self.pads.colon_len()
        } else {
            0
        }) + (if self.middle_comment_length > 0 {
            self.middle_comment_length + self.pads.comment_len()
        } else {
            0
        }) + self.composite_value_length
            + if self.postfix_comment_length > 0 {
                self.postfix_comment_length + self.pads.comment_len()
            } else {
                0
            };
    }

    fn get_template_complexity(&self) -> usize {
        if self.children.is_empty() {
            return 0;
        }
        let max_child = self
            .children
            .iter()
            .map(|ch| ch.get_template_complexity())
            .max()
            .unwrap_or(0);
        1 + max_child
    }

    fn get_number_field_width(&self) -> usize {
        if matches!(
            self.number_list_alignment,
            NumberListAlignment::Normalize | NumberListAlignment::Decimal
        ) {
            let raw_dec_len = if self.max_dig_after_dec > 0 { 1 } else { 0 };
            return self.max_dig_before_dec + raw_dec_len + self.max_dig_after_dec;
        }
        self.max_value_length
    }
}

fn dot_or_e_index(value: &str) -> Option<usize> {
    value.find(['.', 'e', 'E'])
}

fn is_truly_zero(value: &str) -> bool {
    let mut chars = value.chars();
    if let Some('-') = chars.clone().next() {
        chars.next();
    }
    let mut saw_any = false;
    for ch in chars {
        if ch == 'e' || ch == 'E' {
            return saw_any;
        }
        if ch != '0' && ch != '.' {
            return false;
        }
        saw_any = true;
    }
    saw_any
}

fn contains_duplicate_keys(list: &[JsonItem]) -> bool {
    let mut seen = std::collections::HashSet::new();
    for item in list {
        if !seen.insert(item.name.clone()) {
            return true;
        }
    }
    false
}
