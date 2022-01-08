use general::Span;

#[derive(Debug, PartialEq)]
pub enum SyntaxError {
    UnexpectedToken {
        expected: Option<Vec<ExpectedToken>>,
        got: Span,
    },
    UnexpectedEOF {
        ctx: EOFContext,
    },
    ExpectedExpression {
        span: Span,
        reason: ExpressionReason,
    },
}

#[derive(Debug, PartialEq)]
pub enum EOFContext {
    Type,
    Identifier,
    Statement,
    Expression,
}

#[derive(Debug, PartialEq)]
pub enum ExpressionReason {
    Conditional,
    Assignment,
    Operand,
    Other(String),
}

#[derive(Debug, PartialEq)]
pub enum ExpectedToken {
    /// ;
    Semicolon,
    /// ,
    Comma,
    /// Some Identifier
    Identifier,
    /// Some Operator
    Operator,
    /// (
    OpenParen,
    /// )
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    /// ]
    CloseBracket,
    /// :
    Colon,
    /// if
    If,
    Literal,
    IntegerLiteral,
    Assignment,
}
