use std::{path::PathBuf, str::FromStr};

use ariadne::{Label, Report, ReportKind, Source};
use compiler::{run, Error};
use preprocessor::loader::files::FileLoader;
use syntax::SyntaxError;

fn main() {
    let source_file = "./tests/files/final.c";

    let mut loader = FileLoader::new();
    loader.add_lib_root(
        PathBuf::from_str("/Library/Developer/CommandLineTools/usr/include/c++/v1/").unwrap(),
    );

    match run(source_file, loader) {
        Ok(_) => {
            println!("Compiled Program");
        }
        Err(e) => match e {
            Error::Preprocessor(pe) => {
                dbg!(pe);
            }
            Error::Syntax(se) => {
                dbg!(&se);
                match se {
                    SyntaxError::UnexpectedEOF => {
                        dbg!("EOF");
                    }
                    SyntaxError::UnexpectedToken { got, expected } => {
                        let content = got.source().content();
                        let content_area = got.source_area();

                        Report::build(ReportKind::Error, (), 0)
                            .with_message("Unexpected Token")
                            .with_label(
                                Label::new(content_area.clone()).with_message("This was given"),
                            )
                            .with_label(
                                Label::new(content_area.clone())
                                    .with_message(format!("Expected {:?}", expected)),
                            )
                            .finish()
                            .print(Source::from(content))
                            .unwrap();
                    }
                }
            }
        },
    };
}
