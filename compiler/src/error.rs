use std::fmt::Debug;

use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind};
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

                        let unknown_c = Color::Red;

                        let identifier_str = name.0.data.fg(unknown_c);

                        Report::build(ReportKind::Error, &name.0.span, 0)
                            .with_message(format!("Unknown Identifier \"{}\" used", identifier_str))
                            .with_label(
                                Label::new((&name.0.span, content_area.clone()))
                                    .with_message("Unknown Identifier")
                                    .with_color(unknown_c),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::UnknownStructField {
                        field_name,
                        struct_def,
                    } => {
                        let sources = SourceCache::from([&struct_def.span, &field_name.0.span]);

                        let mut color_gen = ColorGenerator::new();

                        let struct_c = color_gen.next();
                        let field_name_c = color_gen.next();

                        Report::build(ReportKind::Error, &field_name.0.span, 0)
                            .with_message(format!(
                                "Unknown Field \"{}\" on Struct",
                                field_name.0.data.fg(field_name_c),
                            ))
                            .with_label(
                                Label::new((
                                    &struct_def.span,
                                    struct_def.span.source_area().clone(),
                                ))
                                .with_message("Struct Definition")
                                .with_color(struct_c),
                            )
                            .with_label(
                                Label::new((
                                    &field_name.0.span,
                                    field_name.0.span.source_area().clone(),
                                ))
                                .with_message("Unknown Field")
                                .with_color(field_name_c),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::MismatchedTypes { expected, received } => {
                        let sources = SourceCache::from([&expected.span, &received.span]);

                        let mut color_gen = ColorGenerator::new();

                        let expected_c = color_gen.next();
                        let received_c = color_gen.next();

                        let expected_str = format!("{:?}", expected.data).fg(expected_c);
                        let received_str = format!("{:?}", received.data).fg(received_c);

                        Report::build(ReportKind::Error, &received.span, 0)
                            .with_message(format!(
                                "Type mismatch between {} and {}",
                                expected_str, received_str
                            ))
                            .with_label(
                                Label::new((&expected.span, expected.span.source_area().clone()))
                                    .with_message(format!("Expected {}", expected_str)).with_color(expected_c),
                            )
                            .with_label(
                                Label::new((&received.span, received.span.source_area().clone()))
                                    .with_message(format!("Received {}", received_str)).with_color(received_c),
                            )
                            .with_note("Consider changing either of the Types, to match, or performing an explicit Cast")
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::StructAccessOnNonStruct {
                        field_name,
                        received,
                    } => {
                        let sources = SourceCache::from([&received.span, &field_name.0.span]);

                        let mut color_gen = ColorGenerator::new();

                        let received_c = color_gen.next();
                        let field_c = color_gen.next();

                        Report::build(ReportKind::Error, &received.span, 0)
                            .with_message(format!(
                                "Tried to access a StructField \"{}\" on a Non-Struct Type",
                                field_name.0.data.fg(field_c),
                            ))
                            .with_label(
                                Label::new((&received.span, received.span.source_area().clone()))
                                    .with_message("This is not a Struct Type")
                                    .with_color(received_c),
                            )
                            .with_label(
                                Label::new((
                                    &field_name.0.span,
                                    field_name.0.span.source_area().clone(),
                                ))
                                .with_message("Tried accessing this Field")
                                .with_color(field_c),
                            )
                            .finish()
                            .print(sources)
                            .unwrap();
                    }
                    SemanticError::InvalidType {} => {
                        todo!("Invalid Type")
                    }
                    SemanticError::Redeclaration {
                        name,
                        previous_declaration,
                    } => {
                        dbg!(&name, &previous_declaration);

                        let sources = SourceCache::from([&previous_declaration, &name.0.span]);

                        Report::build(ReportKind::Error, &previous_declaration, 0)
                            .with_message(format!("{:?} was declared again", name.0.data))
                            .with_label(
                                Label::new((
                                    &previous_declaration,
                                    previous_declaration.source_area().clone(),
                                ))
                                .with_message("Previously declared here"),
                            )
                            .with_label(
                                Label::new((&name.0.span, name.0.span.source_area().clone()))
                                    .with_message("Was redeclared here"),
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

                        todo!("Redefinition");
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
