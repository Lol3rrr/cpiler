use std::{iter::Peekable, sync::Arc};

use general::{CharIndexIter, Span};

use crate::{
    state::{Environment, TokenizeState},
    Token, TokenData,
};

pub struct TokenIter {
    span: Arc<Span>,
    chars: Peekable<CharIndexIter<Arc<Span>>>,

    state: TokenizeState,
    last_char: char,
    done: bool,
}

impl TokenIter {
    pub fn new(content: Span) -> Self {
        let tmp = Arc::new(content);
        Self {
            span: tmp.clone(),
            chars: CharIndexIter::new(tmp).peekable(),

            state: TokenizeState::new(),
            last_char: '\n',
            done: false,
        }
    }

    fn is_seperator(tmp: char) -> bool {
        match tmp {
            ' ' | '\t' | '\n' | '(' | ')' | '{' | '}' | '[' | ']' | ';' | '.' | ',' | ':' | '?'
            | '=' | '+' | '-' | '*' | '/' | '%' | '!' | '&' | '|' | '^' | '<' | '>' => true,
            _ => false,
        }
    }
}

impl Iterator for TokenIter {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result;
        let mut n_last_char;
        loop {
            let (index, element) = match self.chars.next() {
                Some(c) => c,
                None => {
                    let end = self.span.content().len();

                    let sub_span = self.state.current_sub(&self.span, end).unwrap();

                    self.done = true;
                    return Token::parse(&sub_span);
                }
            };
            n_last_char = element;

            match (element, self.state.env()) {
                (' ', Environment::Code)
                | ('\n', Environment::Code)
                | ('\t', Environment::Code) => {
                    let next_index = match self.chars.peek() {
                        Some((i, _)) => *i,
                        _ => index + 1,
                    };

                    self.state.move_start(next_index);
                }
                (elem, Environment::Code) if Self::is_seperator(elem) => {
                    // Check if this starts a comment
                    if element == '/' {
                        match self.chars.peek() {
                            Some((n_index, '/')) => {
                                self.state.move_start(*n_index + 1);
                                self.state.switch_env(Environment::SLComment);
                                continue;
                            }
                            Some((n_index, '*')) => {
                                self.state.move_start(*n_index + 1);
                                self.state.switch_env(Environment::MLComment);
                                continue;
                            }
                            _ => {}
                        };
                    }

                    let next_index = match (element, self.chars.peek()) {
                        ('-', Some((index, '>')))
                        | ('+', Some((index, '+')))
                        | ('-', Some((index, '-')))
                        | ('!', Some((index, '=')))
                        | ('=', Some((index, '=')))
                        | ('>', Some((index, '=')))
                        | ('<', Some((index, '=')))
                        | ('&', Some((index, '&')))
                        | ('|', Some((index, '|')))
                        | ('+', Some((index, '=')))
                        | ('-', Some((index, '=')))
                        | ('*', Some((index, '=')))
                        | ('/', Some((index, '=')))
                        | ('%', Some((index, '=')))
                        | ('&', Some((index, '=')))
                        | ('|', Some((index, '=')))
                        | ('^', Some((index, '='))) => {
                            let res = *index + 1;
                            let _ = self.chars.next();
                            res
                        }
                        ('<', Some((index, '<'))) | ('>', Some((index, '>'))) => {
                            let prev_res = *index + 1;

                            let _ = self.chars.next();
                            match self.chars.peek() {
                                Some((n_index, '=')) => {
                                    let res = *n_index + 1;
                                    let _ = self.chars.next();
                                    res
                                }
                                _ => prev_res,
                            }
                        }
                        (_, Some((n_index, _))) => *n_index,
                        _ => index + 1,
                    };

                    let sub_span = self.state.current_sub(&self.span, next_index).unwrap();
                    self.state.move_start(next_index);

                    if let Some(token) = Token::parse(&sub_span) {
                        result = Some(token);
                        break;
                    }
                }
                ('#', Environment::Code) => {
                    if self.last_char == '\n' {
                        self.state.move_start(index + 1);
                        self.state.switch_env(Environment::CompilerDirective);
                    } else {
                        let next_index = index + 1;
                        let sub_span = self.state.current_sub(&self.span, next_index).unwrap();
                        self.state.move_start(next_index);

                        if let Some(token) = Token::parse(&sub_span) {
                            result = Some(token);
                            break;
                        }
                    }
                }
                ('\n', Environment::CompilerDirective) => {
                    let sub_span = self.state.current_sub(&self.span, index).unwrap();

                    self.state.move_start(index + 1);
                    self.state.switch_env(Environment::Code);

                    let content = sub_span.content().to_owned();
                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::CompilerDirective { content },
                    };

                    result = Some(token);
                    break;
                }
                ('"', Environment::Code) => {
                    self.state.move_start(index + 1);
                    self.state.switch_env(Environment::StringLiteral);
                }
                ('"', Environment::StringLiteral) => {
                    let sub_span = self.state.current_sub(&self.span, index).unwrap();

                    self.state.move_start(index + 1);
                    self.state.switch_env(Environment::Code);

                    let content = sub_span.content().to_owned();
                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::StringLiteral { content },
                    };

                    result = Some(token);
                    break;
                }
                ('\n', Environment::SLComment) => {
                    let sub_span = self.state.current_sub(&self.span, index).unwrap();
                    let content = sub_span.content().to_owned();
                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::Comment { content },
                    };

                    self.state.move_start(index + 1);
                    self.state.switch_env(Environment::Code);

                    result = Some(token);
                    break;
                }
                ('*', Environment::MLComment) => {
                    let next_index = match self.chars.peek() {
                        Some((i, '/')) => {
                            let tmp = *i;
                            let _ = self.chars.next();

                            tmp
                        }
                        _ => continue,
                    };

                    let sub_span = self.state.current_sub(&self.span, index).unwrap();
                    let content = sub_span.content().to_owned();

                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::Comment { content },
                    };

                    self.state.move_start(next_index + 1);
                    self.state.switch_env(Environment::Code);

                    result = Some(token);
                    break;
                }
                (_, Environment::Code) => {
                    let end_index = match self.chars.peek() {
                        Some((index, '"')) => *index,
                        Some((index, '#')) => *index,
                        Some((index, tmp)) if Self::is_seperator(*tmp) => *index,
                        _ => continue,
                    };

                    let sub_span = self
                        .state
                        .current_sub(self.span.as_ref(), end_index)
                        .unwrap();

                    if let Some(token) = Token::parse(&sub_span) {
                        result = Some(token);
                        self.state.move_start(end_index);

                        break;
                    }
                }
                _ => {}
            };

            self.last_char = n_last_char;
        }
        self.last_char = n_last_char;

        result
    }
}

#[cfg(test)]
mod tests {
    use general::Source;

    use super::*;

    #[test]
    fn macro_concat_sequence() {
        let input_source = Source::new("test", "a##b");
        let input_span: Span = input_source.clone().into();

        let expected = vec![
            Token {
                span: Span::new_source(input_source.clone(), 0..1),
                data: TokenData::Literal {
                    content: "a".to_string(),
                },
            },
            Token {
                span: Span::new_source(input_source.clone(), 1..2),
                data: TokenData::Hashtag,
            },
            Token {
                span: Span::new_source(input_source.clone(), 2..3),
                data: TokenData::Hashtag,
            },
            Token {
                span: Span::new_source(input_source.clone(), 3..4),
                data: TokenData::Literal {
                    content: "b".to_string(),
                },
            },
        ];

        let result_iter = TokenIter::new(input_span);
        let result_vec: Vec<_> = result_iter.collect();

        assert_eq!(expected, result_vec);
    }
}
