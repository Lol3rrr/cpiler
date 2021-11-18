use general::{Span, SpanRef};

fn split_head_body<'rs, 's, 'o>(raw: &'rs SpanRef<'s>) -> Option<(SpanRef<'o>, SpanRef<'o>)>
where
    'rs: 'o,
    's: 'o,
{
    let mut in_paran = false;
    for (index, tmp) in raw.content().chars().enumerate() {
        match (tmp, in_paran) {
            ('(', false) => {
                in_paran = true;
            }
            (')', true) => {
                in_paran = false;
            }
            (' ', false) => {
                let first = raw.sub_span(0..index).unwrap();
                let second = raw.sub_span(index + 1..raw.content().len()).unwrap();
                return Some((first, second));
            }
            _ => {}
        };
    }

    None
}

fn split_function_head<'i, 'o>(raw: &'i str) -> Option<(&'o str, Vec<String>)>
where
    'i: 'o,
{
    let start_args = raw.find('(').unwrap();
    let end_args = raw.find(')').unwrap();

    let name = &raw[..start_args];

    let arg_body = &raw[start_args + 1..end_args];

    let args = arg_body
        .split(',')
        .map(|a| a.trim())
        .map(|a| a.to_string())
        .filter(|a| !a.is_empty())
        .collect();

    Some((name, args))
}

#[derive(Debug, PartialEq)]
pub enum DefineDirective {
    Block {
        name: String,
        body: Span,
    },
    Function {
        name: String,
        arguments: Vec<String>,
        body: Span,
    },
}

#[derive(Debug, PartialEq)]
pub enum ParseDefine {
    InvalidHeadFormat,
    InvalidFunctionFormat,
}

pub fn parse_define<'s, C>(content: C) -> Result<DefineDirective, ParseDefine>
where
    C: Into<SpanRef<'s>>,
{
    let content_span = content.into();
    let (head, body) = split_head_body(&content_span).unwrap_or_else(|| {
        let size = content_span.content().len();
        let head = content_span.sub_span(0..size).unwrap();
        let body = content_span.sub_span(size..size).unwrap();
        (head, body)
    });

    if head.content().contains('(') {
        let (name, arguments) =
            split_function_head(head.content()).ok_or(ParseDefine::InvalidFunctionFormat)?;

        Ok(DefineDirective::Function {
            name: name.to_string(),
            arguments,
            body: body.into(),
        })
    } else {
        let name = head;

        Ok(DefineDirective::Block {
            name: name.content().to_string(),
            body: body.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use general::Source;

    use super::*;

    #[test]
    fn valid_define_block() {
        let source = Source::new("tmp", "TEST 0");
        let content: Span = source.clone().into();

        let expected = Ok(DefineDirective::Block {
            name: "TEST".to_string(),
            body: Span::new_source(source.clone(), 5..6),
        });

        let result = parse_define(&content);

        assert_eq!(expected, result);
    }

    #[test]
    fn valid_define_function_0args() {
        let source = Source::new("tmp", "TEST() (13)");
        let content: Span = source.clone().into();

        let expected = Ok(DefineDirective::Function {
            name: "TEST".to_string(),
            arguments: Vec::new(),
            body: Span::new_source(source.clone(), 7..11),
        });

        let result = parse_define(&content);

        assert_eq!(expected, result);
    }

    #[test]
    fn valid_empty_define() {
        let source = Source::new("tmp", "TEST");
        let content: Span = source.clone().into();

        let expected = Ok(DefineDirective::Block {
            name: "TEST".to_string(),
            body: Span::new_source(source.clone(), 4..4),
        });

        let result = parse_define(&content);

        assert_eq!(expected, result);
    }
}
