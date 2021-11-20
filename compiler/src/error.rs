use std::fmt::Debug;

use ariadne::{Label, Report, ReportKind, Source};
use syntax::SyntaxError;

#[derive(Debug)]
pub enum Error<P> {
    Preprocessor(preprocessor::ProcessError<P>),
    Syntax(syntax::SyntaxError),
}

impl<P> Error<P>
where
    P: Debug,
{
    pub fn display(self) {
        match self {
            Self::Preprocessor(pe) => {
                dbg!(pe);
            }
            Self::Syntax(se) => {
                match se {
                    SyntaxError::UnexpectedEOF => {
                        dbg!("EOF");
                    }
                    SyntaxError::UnexpectedToken { got, expected } => {
                        let content = got.source().content();
                        let content_area = got.source_area();

                        Report::build(ReportKind::Error, (), 0)
                            .with_message("Syntax Error: Unexpected Token")
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
                    SyntaxError::ExpectedExpression { span, reason } => {
                        dbg!(&reason);

                        let content = span.source().content();
                        let content_area = span.source_area();

                        Report::build(ReportKind::Error, (), 0)
                            .with_message("Expected Expression")
                            .with_label(
                                Label::new(content_area.clone()).with_message("Because of this"),
                            )
                            .finish()
                            .print(Source::from(content))
                            .unwrap();
                    }
                };
            }
        };
    }
}
