use std::fmt::Debug;

use ariadne::{sources, Label, Report, ReportKind, Source};
use semantic::SemanticError;
use syntax::SyntaxError;

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
                        let content = got.source().content();
                        let content_area = got.source_area();

                        let source_name = got.source().name().to_string();

                        let source_list = sources(vec![(source_name.clone(), content.to_string())]);

                        Report::build(ReportKind::Error, source_name.clone(), 0)
                            .with_message("Syntax Error: Unexpected Token")
                            .with_label(
                                Label::new((source_name.clone(), content_area.clone()))
                                    .with_message("This was given"),
                            )
                            .with_label(
                                Label::new((source_name.clone(), content_area.clone()))
                                    .with_message(format!("Expected {:?}", expected)),
                            )
                            .finish()
                            .print(source_list)
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
            Self::Semantic(se) => {
                match se {
                    SemanticError::UnknownIdentifier { name } => {
                        let content = name.0.span.source().content();
                        let content_area = name.0.span.source_area();

                        Report::build(ReportKind::Error, (), 0)
                            .with_message(format!("Unknown Identifier \"{}\" used", name.0.data))
                            .with_label(Label::new(content_area.clone()).with_message("Unknown"))
                            .finish()
                            .print(Source::from(content))
                            .unwrap();
                    }
                    SemanticError::MismatchedTypes { expected, received } => {
                        let received_span = received.span;
                        let received_source = received_span.source();
                        let received_area = received_span.source_area();

                        let expected_span = expected.span;
                        let expected_source = expected_span.source();
                        let expected_area = expected_span.source_area();

                        let print_sources_vec = {
                            let mut tmp = vec![(
                                expected_source.name().to_string(),
                                expected_source.content().to_string(),
                            )];
                            if received_source.name() != expected_source.name() {
                                tmp.push((
                                    received_source.name().to_string(),
                                    received_source.content().to_string(),
                                ));
                            }

                            tmp
                        };
                        let print_sources = sources(print_sources_vec);

                        Report::build(ReportKind::Error, received_source.name().to_string(), 0)
                            .with_message(format!(
                                "Type mismatch between {:?} and {:?}",
                                expected.data, received.data
                            ))
                            .with_label(
                                Label::new((
                                    expected_source.name().to_string(),
                                    expected_area.clone(),
                                ))
                                .with_message(format!("Expected {:?}", expected.data)),
                            )
                            .with_label(
                                Label::new((
                                    received_source.name().to_string(),
                                    received_area.clone(),
                                ))
                                .with_message(format!("Received {:?}", received.data)),
                            )
                            .finish()
                            .print(print_sources)
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
