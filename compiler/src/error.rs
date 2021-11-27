use std::fmt::Debug;

use ariadne::{Label, Report, ReportKind};
use semantic::SemanticError;
use syntax::SyntaxError;

mod cache;
use cache::*;

#[derive(Debug)]
pub enum Error<P> {
    Preprocessor(preprocessor::ProcessError<P>),
    Syntax(syntax::SyntaxError),
    Semantic(semantic::SemanticError),
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
                        let content_area = got.source_area();

                        let sources = SourceCache::from([&got]);

                        Report::build(ReportKind::Error, &got, 0)
                            .with_message("Syntax Error: Unexpected Token")
                            .with_label(
                                Label::new((&got, content_area.clone()))
                                    .with_message("This was given"),
                            )
                            .with_label(
                                Label::new((&got, content_area.clone()))
                                    .with_message(format!("Expected {:?}", expected)),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SyntaxError::ExpectedExpression { span, reason } => {
                        dbg!(&reason);

                        let content_area = span.source_area();

                        let sources = SourceCache::from([&span]);

                        Report::build(ReportKind::Error, &span, 0)
                            .with_message("Expected Expression")
                            .with_label(
                                Label::new((&span, content_area.clone()))
                                    .with_message("Because of this"),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                };
            }
            Self::Semantic(se) => {
                match se {
                    SemanticError::UnknownIdentifier { name } => {
                        let content_area = name.0.span.source_area();

                        let sources = SourceCache::from([&name.0.span]);

                        Report::build(ReportKind::Error, &name.0.span, 0)
                            .with_message(format!("Unknown Identifier \"{}\" used", name.0.data))
                            .with_label(
                                Label::new((&name.0.span, content_area.clone()))
                                    .with_message("Unknown"),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::MismatchedTypes { expected, received } => {
                        let mut sources = SourceCache::from([&expected.span]);
                        if expected.span.source().name() != received.span.source().name() {
                            sources.add_source(&received.span);
                        }

                        Report::build(ReportKind::Error, &received.span, 0)
                            .with_message(format!(
                                "Type mismatch between {:?} and {:?}",
                                expected.data, received.data
                            ))
                            .with_label(
                                Label::new((&expected.span, expected.span.source_area().clone()))
                                    .with_message(format!("Expected {:?}", expected.data)),
                            )
                            .with_label(
                                Label::new((&received.span, received.span.source_area().clone()))
                                    .with_message(format!("Received {:?}", received.data)),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::Redefinition {
                        name,
                        previous_definition,
                    } => {
                        dbg!(&name, &previous_definition);

                        todo!("Handle redefinition error");
                    }
                    SemanticError::MismatchedFunctionArgsCount { expected, received } => {
                        dbg!(&expected, &received);

                        todo!("Mismatched Function Args Count");
                    }
                };
            }
        };
    }
}
