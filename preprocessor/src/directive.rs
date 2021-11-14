use general::{Span, SpanRef};

mod define;

#[derive(Debug, PartialEq, Clone)]
pub enum ConditionalDirective {
    If { condition: Span },
    IfDef { name: String },
    IfNDef { name: String },
    Else,
    ElseIf { condition: Span },
}

#[derive(Debug, PartialEq, Clone)]
pub enum GnuExtesion {
    /// Docs:
    /// https://gcc.gnu.org/onlinedocs/cpp/Wrapper-Headers.html
    IncludeNext { path: String },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Extensions {
    GNU(GnuExtesion),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Directive {
    Include {
        local: bool,
        path: String,
    },
    DefineBlock {
        name: String,
        body: Span,
    },
    DefineFunction {
        name: String,
        arguments: Vec<String>,
        body: Span,
    },
    Undefine {
        name: String,
    },
    Conditional(ConditionalDirective),
    EndIf,
    Pragma {
        content: Span,
    },
    Extensions(Extensions),
}

#[derive(Debug, PartialEq)]
pub enum ParseDirectiveError {
    InvalidInclude { path: String },
    InvalidDefine(define::ParseDefine),
    UnknownDirective { raw: String },
    InvalidFormat { raw: String },
}

impl Directive {
    pub fn parse<'s>(raw: SpanRef<'s>) -> Result<Self, ParseDirectiveError> {
        let (d_type, body) = match raw.content().find(' ') {
            Some(s_index) => {
                let first = raw.sub_span(0..s_index).unwrap();
                let second = raw.sub_span(s_index + 1..raw.content().len()).unwrap();

                (first, Some(second))
            }
            None => {
                let first = raw;

                (first, None)
            }
        };

        match (d_type.content(), body) {
            ("include", Some(body)) => {
                let body_content = body.content();
                if body_content.starts_with('"') && body_content.ends_with('"') {
                    let raw_path = body_content
                        .strip_prefix('"')
                        .unwrap()
                        .strip_suffix('"')
                        .unwrap();

                    Ok(Self::Include {
                        local: true,
                        path: raw_path.to_owned(),
                    })
                } else if body_content.starts_with('<') && body_content.ends_with('>') {
                    let raw_path = body_content
                        .strip_prefix('<')
                        .unwrap()
                        .strip_suffix('>')
                        .unwrap();

                    Ok(Self::Include {
                        local: false,
                        path: raw_path.to_owned(),
                    })
                } else {
                    Err(ParseDirectiveError::InvalidInclude {
                        path: body_content.to_owned(),
                    })
                }
            }
            // From a GNU Extension that we should probably support as well
            ("include_next", Some(body)) => {
                let mut path = String::new();
                path.reserve(body.content().len());

                body.content().chars().for_each(|c| match c {
                    '"' | '<' | '>' => {}
                    other => {
                        path.push(other);
                    }
                });

                Ok(Directive::Extensions(Extensions::GNU(
                    GnuExtesion::IncludeNext { path },
                )))
            }
            ("define", Some(body)) => {
                let def = match define::parse_define(body) {
                    Ok(d) => d,
                    Err(e) => return Err(ParseDirectiveError::InvalidDefine(e)),
                };

                match def {
                    define::DefineDirective::Block { name, body } => {
                        Ok(Directive::DefineBlock { name, body })
                    }
                    define::DefineDirective::Function {
                        name,
                        arguments,
                        body,
                    } => Ok(Directive::DefineFunction {
                        name,
                        arguments,
                        body,
                    }),
                }
            }
            ("undef", Some(body)) => Ok(Directive::Undefine {
                name: body.content().to_owned(),
            }),
            ("pragma", Some(body)) => Ok(Directive::Pragma {
                content: body.into(),
            }),
            ("if", Some(body)) => Ok(Directive::Conditional(ConditionalDirective::If {
                condition: body.into(),
            })),
            ("ifdef", Some(body)) => Ok(Directive::Conditional(ConditionalDirective::IfDef {
                name: body.content().to_owned(),
            })),
            ("ifndef", Some(body)) => Ok(Directive::Conditional(ConditionalDirective::IfNDef {
                name: body.content().to_owned(),
            })),
            ("else", None) => Ok(Directive::Conditional(ConditionalDirective::Else)),
            ("elif", Some(body)) => Ok(Directive::Conditional(ConditionalDirective::ElseIf {
                condition: body.into(),
            })),
            ("endif", _) => Ok(Directive::EndIf),
            (name, body) => Err(ParseDirectiveError::UnknownDirective {
                raw: name.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_directive() {
        let body = Span::from_parts("tmp", "random other", 0..12);

        let expected = Err(ParseDirectiveError::UnknownDirective {
            raw: "random".to_string(),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }

    #[test]
    fn local_include() {
        let body_content = "include \"testing\"";
        let body = Span::from_parts("tmp", body_content, 0..body_content.len());

        let expected = Ok(Directive::Include {
            local: true,
            path: "testing".to_string(),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }

    #[test]
    fn non_local_include() {
        let body_content = "include <testing>";
        let body = Span::from_parts("tmp", body_content, 0..body_content.len());

        let expected = Ok(Directive::Include {
            local: false,
            path: "testing".to_string(),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }

    #[test]
    fn define_block() {
        let body_content = "define TMP 123";
        let body = Span::from_parts("tmp", body_content, 0..body_content.len());

        let expected = Ok(Directive::DefineBlock {
            name: "TMP".to_string(),
            body: Span::from_parts("tmp", "123", 11..14),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }

    #[test]
    fn define_function() {
        let body_content = "define TMP(x) (x * 10)";
        let body = Span::from_parts("tmp", body_content, 0..body_content.len());

        let expected = Ok(Directive::DefineFunction {
            name: "TMP".to_string(),
            arguments: vec!["x".to_string()],
            body: Span::from_parts("tmp", "(x * 10)", 14..22),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }

    #[test]
    fn simple_undef() {
        let body_content = "undef TMP";
        let body = Span::from_parts("tmp", body_content, 0..body_content.len());

        let expected = Ok(Directive::Undefine {
            name: "TMP".to_string(),
        });

        let result = Directive::parse((&body).into());

        assert_eq!(expected, result);
    }
}
