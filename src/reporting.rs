#![allow(dead_code)]

use std::{
    fmt::Display,
    ops::{Bound, RangeBounds},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: u32::try_from(start).expect("Span start doesn't fit within u32"),
            end: u32::try_from(end).expect("Span end doesn't fit within u32"),
        }
    }

    pub fn start_as_usize(&self) -> usize {
        usize::try_from(self.start).expect("Span start doesn't fit within usize")
    }

    pub fn end_as_usize(&self) -> usize {
        usize::try_from(self.end).expect("Span end doesn't fit within usize")
    }

    pub fn combine_with(&self, other: &Span) -> Option<Span> {
        Some(Self {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        })
    }
}

pub trait Spannable {
    type Slice;

    fn slice<R: RangeBounds<usize>>(&self, bounds: R) -> Option<Self::Slice>;

    fn up_to(&self, end: usize) -> Option<Self::Slice> {
        self.slice(0..end)
    }

    fn starting_at(&self, start: usize) -> Option<Self::Slice> {
        self.slice(start..)
    }
}

impl<'a> Spannable for &'a str {
    type Slice = &'a str;

    fn slice<R: RangeBounds<usize>>(&self, bounds: R) -> Option<Self::Slice> {
        let start_bound = match bounds.start_bound() {
            Bound::Included(num) => *num,
            Bound::Excluded(num) => num
                .checked_add(1)
                .expect("Overflow when adding 1 to Spannable slice start"),
            Bound::Unbounded => 0,
        };

        let end_bound = match bounds.end_bound() {
            Bound::Included(num) => num
                .checked_add(1)
                .expect("Overflow when adding 1 to Spannable slice end"),
            Bound::Excluded(num) => *num,
            Bound::Unbounded => str::len(self),
        };

        str::get(self, start_bound..end_bound)
    }
}

const ESC: &str = "\x1B";
const RESET: &str = "[0m";
const NONE: &str = "";

const BLACK: &str = "[38;5;0m";
const RED: &str = "[38;5;1m";
const GREEN: &str = "[38;5;2m";
const YELLOW: &str = "[38;5;3m";
const BLUE: &str = "[38;5;4m";
const MAGENTA: &str = "[38;5;5m";
const CYAN: &str = "[38;5;6m";
const WHITE: &str = "[38;5;7m";

const BRIGHT_BLACK: &str = "[38;5;8m";
const BRIGHT_RED: &str = "[38;5;9m";
const BRIGHT_GREEN: &str = "[38;5;10m";
const BRIGHT_YELLOW: &str = "[38;5;11m";
const BRIGHT_BLUE: &str = "[38;5;12m";
const BRIGHT_MAGENTA: &str = "[38;5;13m";
const BRIGHT_CYAN: &str = "[38;5;14m";
const BRIGHT_WHITE: &str = "[38;5;15m";

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReportColor {
    #[default]
    None,
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl ReportColor {
    pub fn to_ansi_escape(self) -> &'static str {
        match self {
            Self::None => NONE,
            Self::Reset => RESET,
            Self::Black => BLACK,
            Self::Red => RED,
            Self::Green => GREEN,
            Self::Yellow => YELLOW,
            Self::Blue => BLUE,
            Self::Magenta => MAGENTA,
            Self::Cyan => CYAN,
            Self::White => WHITE,
            Self::BrightBlack => BRIGHT_BLACK,
            Self::BrightRed => BRIGHT_RED,
            Self::BrightGreen => BRIGHT_GREEN,
            Self::BrightYellow => BRIGHT_YELLOW,
            Self::BrightBlue => BRIGHT_BLUE,
            Self::BrightMagenta => BRIGHT_MAGENTA,
            Self::BrightCyan => BRIGHT_CYAN,
            Self::BrightWhite => BRIGHT_WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ReportColors {
    pub message: ReportColor,
    pub note: ReportColor,
    pub divider: ReportColor,
    pub trim: ReportColor,
    pub highlight: ReportColor,
    pub underline: ReportColor,
    esc: &'static str,
    reset: &'static str,
}

impl Default for ReportColors {
    fn default() -> Self {
        Self {
            message: ReportColor::BrightRed,
            note: ReportColor::BrightGreen,
            divider: ReportColor::BrightCyan,
            trim: ReportColor::BrightBlue,
            highlight: ReportColor::BrightRed,
            underline: ReportColor::BrightMagenta,
            esc: ESC,
            reset: RESET,
        }
    }
}

impl ReportColors {
    pub fn error() -> Self {
        Self::default()
    }

    pub fn warning() -> Self {
        Self {
            message: ReportColor::BrightYellow,
            highlight: ReportColor::BrightYellow,
            ..Default::default()
        }
    }

    pub fn colorless() -> Self {
        Self {
            message: ReportColor::None,
            note: ReportColor::None,
            divider: ReportColor::None,
            trim: ReportColor::None,
            highlight: ReportColor::None,
            underline: ReportColor::None,
            esc: "",
            reset: "",
        }
    }
}

pub struct ReportData<S, K, N, T>
where
    S: Display,
    K: Display,
    N: Display,
    T: Display,
{
    pub source: S,
    pub kind: K,
    pub name: N,
    pub trimmed: T,
    pub color_data: ReportColors,
    error_messages: Vec<String>,
}

impl<S, K, N, T> ReportData<S, K, N, T>
where
    S: Display,
    K: Display,
    N: Display,
    T: Display,
{
    pub fn new(source: S, kind: K, name: N, trimmed: T, color_data: ReportColors) -> Self {
        Self {
            source,
            kind,
            name,
            trimmed,
            color_data,
            error_messages: vec![],
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.error_messages.is_empty()
    }

    pub fn errors(&self) -> &[String] {
        self.error_messages.as_slice()
    }

    pub fn take_errors(&mut self) -> Vec<String> {
        let mut errors = vec![];
        errors.append(&mut self.error_messages);
        errors
    }
}

const MAX_LINE_SCAN_LENGTH: usize = 40;

pub trait Reportable<Message, Notes>
where
    Message: Display,
    Notes: Display,
{
    fn span(&self) -> Span;

    fn message(&self) -> Option<Message>;

    fn notes(&self) -> Vec<Notes>;

    fn printed_source_map<S>(source: S) -> String
    where
        S: Display,
    {
        source.to_string().replace('\n', "").replace('\t', "    ")
    }

    fn printed_source_length<S>(source: S) -> usize
    where
        S: Display,
    {
        source.to_string().chars().fold(0, |accum, ch| {
            accum
                + match ch {
                    '\t' => 4,
                    '\n' => 0,
                    _ => 1,
                }
        })
    }

    fn report<S, K, N, T>(&self, report_data: &mut ReportData<S, K, N, T>)
    where
        S: Display,
        K: Display,
        N: Display,
        T: Display,
    {
        let source = report_data.source.to_string();
        let kind = Self::printed_source_map(report_data.kind.to_string());
        let name = Self::printed_source_map(report_data.name.to_string());
        let trimmed = Self::printed_source_map(report_data.trimmed.to_string());

        let kind = kind.as_str();
        let trimmed = trimmed.as_str();

        let span_start = self.span().start_as_usize();
        let span_end = self.span().end_as_usize();

        let line_number = source
            .char_indices()
            .take_while(|(i, _ch)| *i < span_start)
            .filter(|(_i, ch)| *ch == '\n')
            .count()
            + 1;

        let line_start = source[..span_start]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map(|(i, _ch)| i)
            .unwrap_or(span_start);

        let column = source[line_start..]
            .char_indices()
            .take_while(|(i, _ch)| line_start + *i < span_start)
            .last()
            .map(|(i, _ch)| i + 1)
            .unwrap_or(1);

        let line_prefix_is_long = line_start <= span_start
            && source[line_start..span_start].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let first_highlight_end = source[span_start..]
            .char_indices()
            .enumerate()
            .skip_while(|(_steps, (_i, ch))| ch.is_ascii_whitespace())
            .take_while(|(steps, (i, ch))| {
                *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH && *i < span_end - span_start
            })
            .last()
            .map(|(_steps, (i, ch))| i + ch.len_utf8() + span_start)
            .unwrap_or(span_end);

        let second_highlight_start = source[..span_end]
            .char_indices()
            .rev()
            .enumerate()
            .skip_while(|(_steps, (_i, ch))| ch.is_ascii_whitespace())
            .take_while(|(steps, (i, ch))| {
                *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH && *i >= span_start
            })
            .last()
            .map(|(_steps, (i, _ch))| i)
            .unwrap_or(span_start);

        let highlight_is_long = second_highlight_start > first_highlight_end;

        let line_end = source[span_end..]
            .char_indices()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map(|(i, ch)| i + ch.len_utf8() + span_end)
            .unwrap_or(span_end);

        let line_suffix_is_long = second_highlight_start <= line_end
            && source[second_highlight_start..line_end].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let last_line_end = source[..line_start]
            .char_indices()
            .rev()
            .skip_while(|(_i, ch)| ch.is_ascii_whitespace())
            .take(1)
            .last()
            .map(|(i, ch)| i + ch.len_utf8())
            .unwrap_or(line_start);

        let last_line_start = source[..last_line_end]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map(|(i, _ch)| i)
            .unwrap_or(last_line_end);

        let last_line_number = line_number
            .checked_sub(if last_line_start < line_start {
                source[last_line_start..line_start]
                    .chars()
                    .filter(|ch| *ch == '\n')
                    .count()
            } else {
                0
            })
            .unwrap_or(line_number);

        let last_line_not_this_line = last_line_end <= line_start
            && source[last_line_end..line_start]
                .chars()
                .any(|ch| ch == '\n');

        let last_line_is_long = last_line_start <= last_line_end
            && source[last_line_start..last_line_end].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let next_line_start = source[line_end..]
            .char_indices()
            .skip_while(|(_i, ch)| ch.is_ascii_whitespace())
            .take(1)
            .last()
            .map(|(i, _ch)| i + line_end)
            .unwrap_or(line_end);

        let next_line_start = source[..next_line_start]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map(|(i, _ch)| i)
            .unwrap_or(next_line_start);

        let next_line_end = source[next_line_start..]
            .char_indices()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map(|(i, ch)| i + ch.len_utf8() + next_line_start)
            .unwrap_or(next_line_start);

        let next_line_number = line_number
            + source[line_start..next_line_start]
                .chars()
                .filter(|ch| *ch == '\n')
                .count();

        let line_number_digits =
            ((next_line_number + 1).checked_ilog10().unwrap_or_default() + 1) as usize;

        let next_line_not_this_line = line_end <= next_line_start
            && source[line_end..next_line_start]
                .chars()
                .any(|ch| ch == '\n');

        let next_line_is_long = next_line_start <= next_line_end
            && source[next_line_start..next_line_end].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let mut buffer = format!(
            "{0}{1}---------------{2}{3}\n{4}{5}{6}:{line_number}:{column}\n{7}{8}{9}:",
            report_data.color_data.esc,
            report_data.color_data.divider.to_ansi_escape(),
            report_data.color_data.esc,
            report_data.color_data.reset,
            report_data.color_data.esc,
            report_data.color_data.message.to_ansi_escape(),
            name,
            report_data.color_data.esc,
            report_data.color_data.message.to_ansi_escape(),
            kind,
        );

        if let Some(expected) = self.message() {
            buffer.push(' ');
            buffer.push_str(Self::printed_source_map(expected).as_str());
        }

        buffer.push_str(report_data.color_data.esc);
        buffer.push_str(report_data.color_data.reset);

        buffer.push('\n');

        let false_start = source[..span_start]
            .char_indices()
            .rev()
            .enumerate()
            .take_while(|(steps, (i, ch))| {
                *ch != '\n'
                    && *i >= line_start
                    && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
            })
            .last()
            .map(|(_steps, (i, _ch))| i)
            .unwrap_or(span_start);

        let false_end = source[span_end..]
            .char_indices()
            .enumerate()
            .take_while(|(steps, (_i, ch))| {
                *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
            })
            .last()
            .map(|(_steps, (i, ch))| span_end + i + ch.len_utf8())
            .unwrap_or(line_end);

        if last_line_not_this_line {
            let false_end = source[last_line_start..]
                .char_indices()
                .enumerate()
                .take_while(|(steps, (_i, ch))| {
                    *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
                })
                .last()
                .map(|(_steps, (i, ch))| last_line_start + i + ch.len_utf8())
                .unwrap_or(last_line_end);

            buffer.push_str(
                format!(
                    " {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    last_line_number,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[last_line_start..false_end]).as_str());

            if last_line_is_long {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push('\n');
        }

        if highlight_is_long
            && source[first_highlight_end..second_highlight_start]
                .chars()
                .any(|ch| ch == '\n')
        {
            buffer.push_str(
                format!(
                    " {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    line_number,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    format!(
                        "{0}{1}{2}{3}{4} ",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(Self::printed_source_map(&source[false_start..span_start]).as_str());

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.highlight.to_ansi_escape(),
                    Self::printed_source_map(&source[span_start..first_highlight_end]),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    " ",
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    " ".repeat(Self::printed_source_length(trimmed) + 1)
                        .as_str(),
                );
            }

            buffer.push_str(
                " ".repeat(Self::printed_source_length(
                    &source[false_start..span_start],
                ))
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.underline.to_ansi_escape(),
                    "^".repeat(
                        Self::printed_source_length(&source[span_start..first_highlight_end])
                            .max(1)
                    ),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    " ",
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.trim.to_ansi_escape(),
                    trimmed,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    line_number
                        + source[first_highlight_end..second_highlight_start]
                            .chars()
                            .filter(|ch| *ch == '\n')
                            .count(),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.highlight.to_ansi_escape(),
                    Self::printed_source_map(&source[second_highlight_start..span_end]).as_str(),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            if line_suffix_is_long && false_end < line_end {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    " ",
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.underline.to_ansi_escape(),
                    "^".repeat(
                        Self::printed_source_length(&source[second_highlight_start..span_end])
                            .max(1)
                    ),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );
        } else if highlight_is_long {
            buffer.push_str(
                format!(
                    " {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    line_number,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    format!(
                        "{0}{1}{2}{3}{4} ",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(Self::printed_source_map(&source[false_start..span_start]).as_str());

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.highlight.to_ansi_escape(),
                    Self::printed_source_map(&source[span_start..first_highlight_end]),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    " {0}{1}{2}{3}{4} ",
                    report_data.color_data.esc,
                    report_data.color_data.trim.to_ansi_escape(),
                    trimmed,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.highlight.to_ansi_escape(),
                    Self::printed_source_map(&source[second_highlight_start..span_end]),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    " ",
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    format!(
                        "{0}{1}{2}{3}{4} ",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        " ".repeat(Self::printed_source_length(trimmed) + 1)
                            .as_str(),
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(
                " ".repeat(Self::printed_source_length(
                    &source[false_start..span_start],
                ))
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.underline.to_ansi_escape(),
                    "^".repeat(
                        Self::printed_source_length(&source[span_start..first_highlight_end])
                            .max(1)
                    ),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    " {0}{1}{2}{3}{4} ",
                    report_data.color_data.esc,
                    report_data.color_data.trim.to_ansi_escape(),
                    " ".repeat(Self::printed_source_length(trimmed) + 2)
                        .as_str(),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.underline.to_ansi_escape(),
                    "^".repeat(
                        Self::printed_source_length(&source[second_highlight_start..span_end])
                            .max(1)
                    ),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );
        } else {
            buffer.push_str(
                format!(
                    " {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    line_number,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    format!(
                        "{0}{1}{2}{3}{4} ",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(Self::printed_source_map(&source[false_start..span_start]).as_str());

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.highlight.to_ansi_escape(),
                    Self::printed_source_map(&source[span_start..span_end]),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            if line_suffix_is_long && false_end < line_end {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(
                format!(
                    "\n {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    " ",
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    " ".repeat(Self::printed_source_length(trimmed) + 1)
                        .as_str(),
                );
            }

            buffer.push_str(
                " ".repeat(Self::printed_source_length(
                    &source[false_start..span_start],
                ))
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    report_data.color_data.esc,
                    report_data.color_data.underline.to_ansi_escape(),
                    "^".repeat(Self::printed_source_length(&source[span_start..span_end]).max(1)),
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );
        }

        buffer.push('\n');

        if next_line_not_this_line {
            let false_end = source[next_line_start..]
                .char_indices()
                .enumerate()
                .take_while(|(steps, (_i, ch))| {
                    *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
                })
                .last()
                .map(|(_steps, (i, ch))| next_line_start + i + ch.len_utf8())
                .unwrap_or(next_line_end);

            buffer.push_str(
                format!(
                    " {0}{1}{3:>2$} | {4}{5}",
                    report_data.color_data.esc,
                    report_data.color_data.divider.to_ansi_escape(),
                    line_number_digits,
                    next_line_number,
                    report_data.color_data.esc,
                    report_data.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[next_line_start..false_end]).as_str());

            if next_line_is_long {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        report_data.color_data.esc,
                        report_data.color_data.trim.to_ansi_escape(),
                        trimmed,
                        report_data.color_data.esc,
                        report_data.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push('\n');
        }

        for note in self.notes().iter() {
            buffer.push_str(report_data.color_data.esc);
            buffer.push_str(report_data.color_data.note.to_ansi_escape());

            buffer.push_str("NOTE: ");
            buffer.push_str(note.to_string().as_str());

            buffer.push_str(report_data.color_data.esc);
            buffer.push_str(report_data.color_data.reset);
            buffer.push('\n');
        }

        report_data.error_messages.push(buffer);
    }
}
