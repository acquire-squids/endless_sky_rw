mod arena;
mod data;
mod lex;
mod macros;
mod parse;
mod reporting;

pub mod prelude {
    pub use crate::data::{Data, Node, NodeIndex, SourceIndex};
    pub use crate::lex::token::{Token, TokenKind};
    pub use crate::reporting::Span;
}

pub use self::prelude::*;

use crate::parse::Parser;
use crate::reporting::{ReportColors, ReportData, Reportable};

use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
};

const EXTENSION: &str = "txt";

pub fn read_path<T: Into<PathBuf>>(path: T) -> Option<DataFolder> {
    let base_path = T::into(path);

    let mut paths = vec![];
    let mut sources = vec![];

    let file_path = base_path.clone();

    if let ReadResult::Ok = read_source(file_path, &mut paths, &mut sources) {
        let reader = Reader::new(paths, sources);

        match reader.read(&mut io::stdout(), true) {
            Ok(data) => return Some(data),
            Err(error) => {
                eprintln!("{error}");
                eprintln!("Failed to read input: see above");
            }
        }
    }

    None
}

#[cfg(all(target_family = "wasm", target_os = "unknown"))]
pub fn read_upload(paths: Vec<String>, sources: Vec<String>) -> Option<(DataFolder, Vec<u8>)> {
    let mut error_buffer = vec![];

    let paths = paths
        .into_iter()
        .map(|p| PathBuf::from(p))
        .collect::<Vec<_>>();

    let reader = Reader::new(paths, sources);

    match reader.read(&mut error_buffer, false) {
        Ok(data) => Some((data, error_buffer)),
        Err(_error) => None,
    }
}

enum ReadResult {
    Ok,
    Err,
}

fn read_source(
    file_path: PathBuf,
    paths: &mut Vec<PathBuf>,
    sources: &mut Vec<String>,
) -> ReadResult {
    if !file_path.exists() {
        eprintln!("File \"{}\" does not exist", file_path.display());
        ReadResult::Err
    } else if file_path.is_dir() {
        let mut all_success = true;

        if let Ok(dir) = fs::read_dir(&file_path) {
            for entry in dir.flatten() {
                let file_path = entry.path();

                all_success &= matches!(read_source(file_path, paths, sources), ReadResult::Ok);
            }

            if all_success {
                ReadResult::Ok
            } else {
                ReadResult::Err
            }
        } else {
            eprintln!("Failed to read directory \"{}\"", file_path.display());
            ReadResult::Err
        }
    } else if file_path.is_file() {
        if matches!(file_path.extension(), Some(ext) if matches!(ext.to_str(), Some(ext) if ext == EXTENSION))
        {
            match fs::read_to_string(&file_path) {
                Ok(source) => {
                    paths.push(file_path);
                    sources.push(source);

                    ReadResult::Ok
                }
                Err(error) => {
                    eprintln!("{error}");
                    eprintln!("Failed to read file (see above)");

                    ReadResult::Err
                }
            }
        } else {
            ReadResult::Ok
        }
    } else {
        eprintln!(
            "Path \"{}\" was not a file or a directory",
            file_path.display()
        );

        ReadResult::Err
    }
}

pub struct DataFolder {
    paths: HashMap<SourceIndex, PathBuf>,
    data: Data,
}

impl DataFolder {
    pub fn path_from_source_index(&self, source_index: SourceIndex) -> Option<&PathBuf> {
        self.paths.get(&source_index)
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}

struct Reader {
    paths: Vec<PathBuf>,
    sources: Vec<SourceIndex>,
    data: Data,
}

impl Reader {
    fn new(paths: Vec<PathBuf>, sources: Vec<String>) -> Self {
        let mut data = Data::default();

        let sources = sources
            .into_iter()
            .map(|source| data.insert_source(source))
            .collect::<Vec<_>>();

        Reader {
            paths,
            sources,
            data,
        }
    }

    fn read<T: Write>(mut self, output: &mut T, colored_errors: bool) -> io::Result<DataFolder> {
        let mut reports = vec![];

        for (i, &source_index) in self.sources.iter().enumerate() {
            let mut parser = Parser::new(source_index);

            parser.parse(&mut self.data);

            let errors = parser.take_errors();

            if !errors.is_empty() {
                let mut report_data = ReportData::new(
                    self.data.get_source(source_index).unwrap().to_owned(),
                    "ERROR",
                    self.paths.get(i).unwrap().display(),
                    "[snip]",
                    if colored_errors {
                        ReportColors::default()
                    } else {
                        ReportColors::colorless()
                    },
                );

                for error in errors {
                    error.report(&mut report_data);
                }

                reports.push(report_data);
            }
        }

        if !reports.is_empty() {
            for report_data in reports.iter_mut() {
                for error in report_data.take_errors() {
                    write!(output, "{}", error)?;
                }
            }
        }

        Ok(DataFolder {
            paths: self
                .sources
                .into_iter()
                .zip(self.paths)
                .collect::<HashMap<_, _>>(),
            data: self.data,
        })
    }
}
