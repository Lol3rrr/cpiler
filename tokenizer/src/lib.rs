// TODO
// Switch to using own custom Span-Iterator that makes it easier to iterate
// over a span and create subspans on it

use general::{Span, SpanData};

mod state;
use state::{Environment, TokenizeState};

mod tokendata;
pub use tokendata::*;

pub type Token = SpanData<TokenData>;

pub fn tokenize(content: Span) -> Vec<Token> {
    let mut result = Vec::new();

    let mut state = TokenizeState::new();

    let last_index = content.content().len();

    let mut iter = content.content().char_indices().peekable();
    let mut prev_char = Some('\n');

    while let Some((index, element)) = iter.next() {
        match (element, state.env()) {
            (' ', Environment::Code)
            | ('\t', Environment::Code)
            | ('(', Environment::Code)
            | (')', Environment::Code)
            | ('{', Environment::Code)
            | ('}', Environment::Code)
            | ('[', Environment::Code)
            | (']', Environment::Code)
            | ('\n', Environment::Code)
            | (';', Environment::Code)
            | (',', Environment::Code)
            | (':', Environment::Code)
            | ('?', Environment::Code)
            | ('=', Environment::Code)
            | ('+', Environment::Code)
            | ('-', Environment::Code)
            | ('*', Environment::Code)
            | ('/', Environment::Code)
            | ('%', Environment::Code)
            | ('!', Environment::Code)
            | ('&', Environment::Code)
            | ('|', Environment::Code)
            | ('^', Environment::Code)
            | ('<', Environment::Code)
            | ('>', Environment::Code)
            | ('.', Environment::Code) => {
                if let Some(sub_span) = state.current_sub(&content, index) {
                    if let Some(token) = Token::parse(&sub_span) {
                        result.push(token);
                    }
                }

                state.move_start(index);

                // Check if this starts a comment
                if element == '/' {
                    match iter.peek() {
                        Some((_, '/')) => {
                            state.move_start(index + 2);
                            state.switch_env(Environment::SLComment);
                            continue;
                        }
                        Some((_, '*')) => {
                            state.move_start(index + 2);
                            state.switch_env(Environment::MLComment);
                            continue;
                        }
                        _ => {}
                    };
                }

                let next_index = match (element, iter.peek()) {
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
                        let _ = iter.next();
                        res
                    }
                    ('<', Some((index, '<'))) | ('>', Some((index, '>'))) => {
                        let prev_res = *index + 1;

                        let _ = iter.next();
                        match iter.peek() {
                            Some((n_index, '=')) => {
                                let res = *n_index + 1;
                                let _ = iter.next();
                                res
                            }
                            _ => prev_res,
                        }
                    }
                    _ => index + 1,
                };

                if let Some(sub_span) = state.current_sub(&content, next_index) {
                    if let Some(token) = Token::parse(&sub_span) {
                        result.push(token);
                    }
                }

                state.move_start(next_index);
            }
            ('#', Environment::Code) => {
                if let Some(sub_span) = state.current_sub(&content, index) {
                    if let Some(token) = Token::parse(&sub_span) {
                        result.push(token);
                    }
                }

                state.move_start(index);

                if let Some('\n') = &prev_char {
                    state.move_start(index + 1);
                    state.switch_env(Environment::CompilerDirective);
                } else {
                    if let Some(sub_span) = state.current_sub(&content, index + 1) {
                        if let Some(token) = Token::parse(&sub_span) {
                            result.push(token);
                        }
                    }

                    state.move_start(index + 1);
                }
            }
            ('\n', Environment::CompilerDirective) => {
                let sub_span = state.current_sub(&content, index).unwrap();

                dbg!(&sub_span);

                state.move_start(index + 1);
                state.switch_env(Environment::Code);

                let content = sub_span.content().to_owned();
                let token = Token {
                    span: sub_span.into(),
                    data: TokenData::CompilerDirective { content },
                };

                result.push(token);
            }
            ('"', Environment::Code) => {
                if let Some(sub_span) = state.current_sub(&content, index) {
                    if let Some(token) = Token::parse(&sub_span) {
                        result.push(token);
                    }
                }

                state.move_start(index + 1);
                state.switch_env(Environment::StringLiteral);
            }
            ('"', Environment::StringLiteral) => {
                let sub_span = state.current_sub(&content, index).unwrap();

                state.move_start(index + 1);
                state.switch_env(Environment::Code);

                let content = sub_span.content().to_owned();
                let token = Token {
                    span: sub_span.into(),
                    data: TokenData::StringLiteral { content },
                };

                result.push(token);
            }
            ('\n', Environment::SLComment) => {
                if let Some(sub_span) = state.current_sub(&content, index) {
                    let content = sub_span.content().to_owned();
                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::Comment { content },
                    };

                    result.push(token);
                }

                state.move_start(index + 1);
                state.switch_env(Environment::Code);
            }
            ('*', Environment::MLComment) => {
                match iter.peek() {
                    Some((_, '/')) => {}
                    _ => continue,
                };
                let _ = iter.next();

                if let Some(sub_span) = state.current_sub(&content, index) {
                    let content = sub_span.content().to_owned();

                    let token = Token {
                        span: sub_span.into(),
                        data: TokenData::Comment { content },
                    };
                    result.push(token);
                }

                state.move_start(index + 2);
                state.switch_env(Environment::Code);
            }
            _ => {}
        };

        prev_char = Some(element);
    }

    if let Some(sub_span) = state.current_sub(&content, last_index) {
        if let Some(token) = Token::parse(&sub_span) {
            result.push(token);
        }
    }

    result
}
