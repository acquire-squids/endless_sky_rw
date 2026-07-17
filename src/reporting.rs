#![allow(dead_code)]

use std::{
    error::Error,
    fmt,
    io::{self, Write},
};

#[derive(Debug)]
pub struct Spanned<T> {
    kind: T,
    span: Span,
}

impl<T> Spanned<T> {
    pub const fn new(kind: T, span: Span) -> Self {
        Self { kind, span }
    }

    pub const fn kind(&self) -> &T {
        &self.kind
    }

    pub const fn span(&self) -> Span {
        self.span
    }

    pub fn transmute<F, U>(self, mut f: F) -> Spanned<U>
    where
        F: FnMut(T) -> U,
    {
        Spanned {
            kind: f(self.kind),
            span: self.span,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    source_id: usize,
    start: usize,
    end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(source_id: usize, start: usize, end: usize) -> Self {
        Self {
            source_id,
            start,
            end,
        }
    }

    #[must_use]
    pub const fn source_id(&self) -> usize {
        self.source_id
    }

    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    #[must_use]
    pub fn lexeme<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get((self.start())..(self.end()))
    }

    #[must_use]
    pub fn line(&self, source: &str) -> Option<usize> {
        source
            .get(..(self.start()))
            .into_iter()
            .flat_map(|text| text.lines())
            .enumerate()
            .last()
            .map(|(line_number, _)| line_number + 1)
    }

    #[must_use]
    pub fn column(&self, source: &str) -> Option<usize> {
        source
            .get(..(self.start()))
            .into_iter()
            .flat_map(|text| text.lines())
            .enumerate()
            .last()
            .into_iter()
            .flat_map(|(_, text)| text.chars().enumerate())
            .last()
            .map(|(column_number, _)| column_number + 1)
    }

    #[must_use]
    pub fn combine_with(&self, other: Self) -> Option<Self> {
        if self.source_id() == other.source_id() {
            Some(Self::new(
                self.source_id(),
                self.start().min(other.start()),
                self.end().max(other.end()),
            ))
        } else {
            None
        }
    }
}

pub const ESC: &str = "\x1B";
pub const BOLD: &str = "[1m";
pub const RESET: &str = "[0m";
pub const NONE: &str = "";

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
    #[must_use]
    pub const fn to_ansi_escape(self) -> &'static str {
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
    kind: ReportColor,
    message: ReportColor,
    divider: ReportColor,
    trim: ReportColor,
    underline: ReportColor,
    esc: &'static str,
    bold: &'static str,
    reset: &'static str,
}

impl Default for ReportColors {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReportData<Source, Kind, Name, Trimmed> {
    source: Source,
    kind: Kind,
    name: Name,
    trimmed: Trimmed,
    color_data: ReportColors,
}

impl<Source, Kind, Name, Trimmed> ReportData<Source, Kind, Name, Trimmed> {
    fn printed_source_map<S>(source: S) -> String
    where
        S: AsRef<str>,
    {
        source.as_ref().replace('\n', "").replace('\t', "    ")
    }

    fn printed_source_length<S>(source: S) -> usize
    where
        S: AsRef<str>,
    {
        source.as_ref().chars().fold(0, |accum, ch| {
            accum
                + match ch {
                    '\t' => 4,
                    '\n' => 0,
                    _ => 1,
                }
        })
    }
}

const MAX_LINE_SCAN_LENGTH: usize = 40;

pub trait Reportable: Error {
    fn notes(&self) -> Vec<String> {
        vec![]
    }
}

impl<Source, Kind, Name, Trimmed> ReportData<Source, Kind, Name, Trimmed>
where
    Source: AsRef<str>,
    Kind: fmt::Display,
    Name: fmt::Display,
    Trimmed: fmt::Display,
{
    // TODO: split this into smaller functions
    #[allow(clippy::too_many_lines)]
    #[allow(clippy::missing_errors_doc)]
    pub fn report<E, W>(&self, report: &Spanned<E>, to: &mut W) -> io::Result<()>
    where
        E: Reportable,
        W: Write,
    {
        let source = self.source.as_ref();
        let kind = Self::printed_source_map(self.kind.to_string());
        let name = Self::printed_source_map(self.name.to_string());
        let trimmed = Self::printed_source_map(self.trimmed.to_string());

        let kind = kind.as_str();
        let trimmed = trimmed.as_str();

        let span_start = report.span().start();
        let span_end = report.span().end();

        let line_number = report.span().line(source).unwrap_or(1);

        let line_start = source[..span_start]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map_or(span_start, |(i, _ch)| i);

        let column = report.span().column(source).unwrap_or(1);

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
            .map_or(span_end, |(_steps, (i, ch))| i + ch.len_utf8() + span_start);

        let second_highlight_start = source[..span_end]
            .char_indices()
            .rev()
            .enumerate()
            .skip_while(|(_steps, (_i, ch))| ch.is_ascii_whitespace())
            .take_while(|(steps, (i, ch))| {
                *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH && *i >= span_start
            })
            .last()
            .map_or(span_start, |(_steps, (i, _ch))| i);

        let highlight_is_long = second_highlight_start > first_highlight_end;

        let line_end = source[span_end..]
            .char_indices()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map_or(span_end, |(i, ch)| i + ch.len_utf8() + span_end);

        let line_suffix_is_long = second_highlight_start <= line_end
            && source[second_highlight_start..line_end].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let last_line_end = source[..line_start]
            .char_indices()
            .rev()
            .skip_while(|(_i, ch)| ch.is_ascii_whitespace())
            .take(1)
            .last()
            .map_or(line_start, |(i, ch)| i + ch.len_utf8());

        let last_line_start = source[..last_line_end]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map_or(last_line_end, |(i, _ch)| i);

        let last_line_number = line_number
            .checked_sub(if last_line_start < line_start {
                source[last_line_start..line_start].lines().count()
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
            .map_or(line_end, |(i, _ch)| i + line_end);

        let next_line_start = source[..next_line_start]
            .char_indices()
            .rev()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map_or(next_line_start, |(i, _ch)| i);

        let next_line_end = source[next_line_start..]
            .char_indices()
            .take_while(|(_i, ch)| *ch != '\n')
            .last()
            .map_or(next_line_start, |(i, ch)| {
                i + ch.len_utf8() + next_line_start
            });

        let next_line_number = line_number + source[line_start..next_line_start].lines().count();

        let line_number_digits =
            ((next_line_number + 1).checked_ilog10().unwrap_or(0) + 1) as usize;

        let next_line_not_this_line = line_end <= next_line_start
            && source[line_end..next_line_start]
                .chars()
                .any(|ch| ch == '\n');

        let next_line_is_long = next_line_start <= next_line_end
            && source[next_line_start..next_line_end].chars().count()
                > MAX_LINE_SCAN_LENGTH + trimmed.chars().count();

        let mut buffer = format!(
            "{}{}{}{}{}{}{}: {}{}{}",
            self.color_data.esc,
            self.color_data.kind.to_ansi_escape(),
            self.color_data.esc,
            self.color_data.bold,
            kind,
            self.color_data.esc,
            self.color_data.message.to_ansi_escape(),
            report.kind(),
            self.color_data.esc,
            self.color_data.reset,
        );

        {
            let mut source = report.kind().source();

            while let Some(cause) = source {
                buffer.push_str(format!("\n    Caused by: {cause}").as_str());
                source = cause.source();
            }
        }

        buffer.push_str(
            format!(
                "\n {}{}{}{}{}-->{}{} {}:{}:{}",
                " ".repeat(line_number_digits),
                self.color_data.esc,
                self.color_data.divider.to_ansi_escape(),
                self.color_data.esc,
                self.color_data.bold,
                self.color_data.esc,
                self.color_data.reset,
                name,
                line_number,
                column,
            )
            .as_str(),
        );

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
            .map_or(span_start, |(_steps, (i, _ch))| i);

        let false_end = source[span_end..]
            .char_indices()
            .enumerate()
            .take_while(|(steps, (_i, ch))| {
                *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
            })
            .last()
            .map_or(line_end, |(_steps, (i, ch))| span_end + i + ch.len_utf8());

        if last_line_not_this_line {
            let false_end = source[last_line_start..]
                .char_indices()
                .enumerate()
                .take_while(|(steps, (_i, ch))| {
                    *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
                })
                .last()
                .map_or(last_line_end, |(_steps, (i, ch))| {
                    last_line_start + i + ch.len_utf8()
                });

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    last_line_number,
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[last_line_start..false_end]).as_str());

            if last_line_is_long {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        self.color_data.esc,
                        self.color_data.trim.to_ansi_escape(),
                        trimmed,
                        self.color_data.esc,
                        self.color_data.reset,
                    )
                    .as_str(),
                );
            }
        }

        buffer.push_str(
            format!(
                "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                self.color_data.esc,
                self.color_data.divider.to_ansi_escape(),
                self.color_data.esc,
                self.color_data.bold,
                line_number_digits,
                line_number,
                self.color_data.esc,
                self.color_data.reset,
            )
            .as_str(),
        );

        if line_prefix_is_long && false_start > line_start {
            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4} ",
                    self.color_data.esc,
                    self.color_data.trim.to_ansi_escape(),
                    trimmed,
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );
        }

        buffer.push_str(Self::printed_source_map(&source[false_start..span_start]).as_str());

        if highlight_is_long
            && source[first_highlight_end..second_highlight_start]
                .chars()
                .any(|ch| ch == '\n')
        {
            buffer.push_str(
                Self::printed_source_map(&source[span_start..first_highlight_end]).as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    " ",
                    self.color_data.esc,
                    self.color_data.reset,
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
                    "{}{}{}{}{}{}{}",
                    self.color_data.esc,
                    self.color_data.underline.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    "^".repeat(
                        Self::printed_source_length(&source[span_start..first_highlight_end])
                            .max(1)
                    ),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    " ",
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{0}{1}{2}{3}{4}",
                    self.color_data.esc,
                    self.color_data.trim.to_ansi_escape(),
                    trimmed,
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    line_number
                        + source[first_highlight_end..second_highlight_start]
                            .lines()
                            .count(),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                Self::printed_source_map(&source[second_highlight_start..span_end]).as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            if line_suffix_is_long && false_end < line_end {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        self.color_data.esc,
                        self.color_data.trim.to_ansi_escape(),
                        trimmed,
                        self.color_data.esc,
                        self.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    " ",
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{}{}{}{}{}{}{}",
                    self.color_data.esc,
                    self.color_data.underline.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    "^".repeat(
                        Self::printed_source_length(&source[second_highlight_start..span_end])
                            .max(1),
                    ),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );
        } else if highlight_is_long {
            buffer.push_str(
                Self::printed_source_map(&source[span_start..first_highlight_end]).as_str(),
            );

            buffer.push_str(
                format!(
                    " {0}{1}{2}{3}{4} ",
                    self.color_data.esc,
                    self.color_data.trim.to_ansi_escape(),
                    trimmed,
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                Self::printed_source_map(&source[second_highlight_start..span_end]).as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    " ",
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            if line_prefix_is_long && false_start > line_start {
                buffer.push_str(
                    format!(
                        "{0}{1}{2}{3}{4} ",
                        self.color_data.esc,
                        self.color_data.trim.to_ansi_escape(),
                        " ".repeat(Self::printed_source_length(trimmed) + 1)
                            .as_str(),
                        self.color_data.esc,
                        self.color_data.reset,
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
                    "{}{}{}{}{}{}{}",
                    self.color_data.esc,
                    self.color_data.underline.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    "^".repeat(
                        Self::printed_source_length(&source[span_start..first_highlight_end])
                            .max(1),
                    ),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    " {0}{1}{2}{3}{4} ",
                    self.color_data.esc,
                    self.color_data.trim.to_ansi_escape(),
                    " ".repeat(Self::printed_source_length(trimmed) + 2)
                        .as_str(),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(
                format!(
                    "{}{}{}{}{}{}{}",
                    self.color_data.esc,
                    self.color_data.underline.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    "^".repeat(
                        Self::printed_source_length(&source[second_highlight_start..span_end])
                            .max(1),
                    ),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );
        } else {
            buffer.push_str(Self::printed_source_map(&source[span_start..span_end]).as_str());

            buffer.push_str(Self::printed_source_map(&source[span_end..false_end]).as_str());

            if line_suffix_is_long && false_end < line_end {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        self.color_data.esc,
                        self.color_data.trim.to_ansi_escape(),
                        trimmed,
                        self.color_data.esc,
                        self.color_data.reset,
                    )
                    .as_str(),
                );
            }

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    " ",
                    self.color_data.esc,
                    self.color_data.reset,
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
                    "{}{}{}{}{}{}{}",
                    self.color_data.esc,
                    self.color_data.underline.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    "^".repeat(Self::printed_source_length(&source[span_start..span_end]).max(1)),
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );
        }

        if next_line_not_this_line {
            let false_end = source[next_line_start..]
                .char_indices()
                .enumerate()
                .take_while(|(steps, (_i, ch))| {
                    *ch != '\n' && *steps < MAX_LINE_SCAN_LENGTH + trimmed.chars().count()
                })
                .last()
                .map_or(next_line_end, |(_steps, (i, ch))| {
                    next_line_start + i + ch.len_utf8()
                });

            buffer.push_str(
                format!(
                    "\n {0}{1}{2}{3}{5:>4$} | {6}{7}",
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    line_number_digits,
                    next_line_number,
                    self.color_data.esc,
                    self.color_data.reset,
                )
                .as_str(),
            );

            buffer.push_str(Self::printed_source_map(&source[next_line_start..false_end]).as_str());

            if next_line_is_long {
                buffer.push_str(
                    format!(
                        " {0}{1}{2}{3}{4}",
                        self.color_data.esc,
                        self.color_data.trim.to_ansi_escape(),
                        trimmed,
                        self.color_data.esc,
                        self.color_data.reset,
                    )
                    .as_str(),
                );
            }
        }

        for note in report.kind().notes().as_slice() {
            buffer.push_str(
                format!(
                    "\n {}{}{}{}{} = {}{}note: {}{}{}",
                    " ".repeat(line_number_digits),
                    self.color_data.esc,
                    self.color_data.divider.to_ansi_escape(),
                    self.color_data.esc,
                    self.color_data.bold,
                    self.color_data.esc,
                    self.color_data.message.to_ansi_escape(),
                    note,
                    self.color_data.esc,
                    self.color_data.reset
                )
                .as_str(),
            );

            buffer.push_str(self.color_data.esc);
            buffer.push_str(self.color_data.reset);
        }

        writeln!(to, "{buffer}")
    }
}

impl ReportColors {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            kind: ReportColor::BrightRed,
            message: ReportColor::BrightWhite,
            divider: ReportColor::BrightBlue,
            trim: ReportColor::BrightGreen,
            underline: ReportColor::BrightMagenta,
            esc: ESC,
            bold: BOLD,
            reset: RESET,
        }
    }

    #[must_use]
    pub const fn colorless() -> Self {
        Self {
            kind: ReportColor::None,
            message: ReportColor::None,
            divider: ReportColor::None,
            trim: ReportColor::None,
            underline: ReportColor::None,
            esc: NONE,
            bold: NONE,
            reset: NONE,
        }
    }

    #[must_use]
    pub const fn error() -> Self {
        Self::new()
    }

    #[must_use]
    pub const fn warning() -> Self {
        Self::new().with_message_color(ReportColor::BrightYellow)
    }

    #[must_use]
    pub const fn with_kind_color(self, color: ReportColor) -> Self {
        Self {
            kind: color,
            message: self.message,
            divider: self.divider,
            trim: self.trim,
            underline: self.underline,
            esc: self.esc,
            bold: self.bold,
            reset: self.reset,
        }
    }

    #[must_use]
    pub const fn with_message_color(self, color: ReportColor) -> Self {
        Self {
            kind: self.kind,
            message: color,
            divider: self.divider,
            trim: self.trim,
            underline: self.underline,
            esc: self.esc,
            bold: self.bold,
            reset: self.reset,
        }
    }

    #[must_use]
    pub const fn with_divider_color(self, color: ReportColor) -> Self {
        Self {
            kind: self.kind,
            message: self.message,
            divider: color,
            trim: self.trim,
            underline: self.underline,
            esc: self.esc,
            bold: self.bold,
            reset: self.reset,
        }
    }

    #[must_use]
    pub const fn with_trim_color(self, color: ReportColor) -> Self {
        Self {
            kind: self.kind,
            message: self.message,
            divider: self.divider,
            trim: color,
            underline: self.underline,
            esc: self.esc,
            bold: self.bold,
            reset: self.reset,
        }
    }

    #[must_use]
    pub const fn with_underline_color(self, color: ReportColor) -> Self {
        Self {
            kind: self.kind,
            message: self.message,
            divider: self.divider,
            trim: self.trim,
            underline: color,
            esc: self.esc,
            bold: self.bold,
            reset: self.reset,
        }
    }
}

impl<Source, Kind, Name, Trimmed> ReportData<Source, Kind, Name, Trimmed> {
    #[must_use]
    pub const fn new(
        source: Source,
        kind: Kind,
        name: Name,
        trimmed: Trimmed,
        color_data: ReportColors,
    ) -> Self {
        Self {
            source,
            kind,
            name,
            trimmed,
            color_data,
        }
    }

    #[must_use]
    pub fn with_source<U>(self, source: U) -> ReportData<U, Kind, Name, Trimmed> {
        ReportData {
            source,
            kind: self.kind,
            name: self.name,
            trimmed: self.trimmed,
            color_data: self.color_data,
        }
    }

    #[must_use]
    pub fn with_kind<U>(self, kind: U) -> ReportData<Source, U, Name, Trimmed> {
        ReportData {
            source: self.source,
            kind,
            name: self.name,
            trimmed: self.trimmed,
            color_data: self.color_data,
        }
    }

    #[must_use]
    pub fn with_name<U>(self, name: U) -> ReportData<Source, Kind, U, Trimmed> {
        ReportData {
            source: self.source,
            kind: self.kind,
            name,
            trimmed: self.trimmed,
            color_data: self.color_data,
        }
    }

    #[must_use]
    pub fn with_trimmed<U>(self, trimmed: U) -> ReportData<Source, Kind, Name, U> {
        ReportData {
            source: self.source,
            kind: self.kind,
            name: self.name,
            trimmed,
            color_data: self.color_data,
        }
    }

    #[must_use]
    pub fn with_color_data(self, color_data: ReportColors) -> Self {
        Self {
            source: self.source,
            kind: self.kind,
            name: self.name,
            trimmed: self.trimmed,
            color_data,
        }
    }
}
