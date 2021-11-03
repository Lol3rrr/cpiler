use std::fmt::Display;

use general::{ParseSpan, SpanRef};

impl ParseSpan for TokenData {
    fn parse<'s>(source: &SpanRef<'s>) -> Option<Self> {
        Some(Self::from(source))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TokenData {
    Keyword(Keyword),
    Assign(Assignment),
    Semicolon,
    Comma,
    QuestionMark,
    Colon,
    Hashtag,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comment { content: String },
    PreProcessorDirective { raw: String },
    Operator(Operator),
    Literal { content: String },
    StringLiteral { content: String },
    CompilerDirective { content: String },
}

impl Display for TokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Keyword(k) => todo!("Format Keyword"),
            Self::Assign(a) => todo!("Format Assignment"),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::QuestionMark => write!(f, "?"),
            Self::Colon => write!(f, ":"),
            Self::Hashtag => write!(f, "#"),
            Self::OpenParen => write!(f, "("),
            Self::CloseParen => write!(f, ")"),
            Self::OpenBrace => write!(f, "{{"),
            Self::CloseBrace => write!(f, "}}"),
            Self::OpenBracket => write!(f, "["),
            Self::CloseBracket => write!(f, "]"),
            Self::Comment { content } => write!(f, "//{}", content),
            Self::PreProcessorDirective { raw } => write!(f, "#{}", raw),
            Self::Operator(op) => todo!("Format Operator"),
            Self::Literal { content } => write!(f, "{}", content),
            Self::StringLiteral { content } => write!(f, "\"{}\"", content),
            Self::CompilerDirective { content } => write!(f, "#{}", content),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Assignment {
    Assign,
    Add,
    Sub,
    Multiply,
    Divide,
    Modulo,
    ShiftLeft,
    ShiftRight,
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Keyword {
    DataType(DataType),
    ControlFlow(ControlFlow),
    Auto,
    Const,
    Default_,
    Bool_,
    Complex_,
    Extern,
    Imaginary_,
    Inline,
    Register,
    Restrict,
    SizeOf,
    Static,
    TypeDef,
    Volatile,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DataType {
    Void,
    Short,
    Char,
    Int,
    Long,
    Float,
    Double,
    Enum,
    Struct,
    Union,
    Unsigned,
    Signed,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ControlFlow {
    If,
    Else,
    Switch,
    Case,
    While,
    For,
    Do,
    Goto,
    Break,
    Continue,
    Return,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    /// +
    Add,
    /// ++
    Increment,
    /// -
    Sub,
    /// --
    Decrement,
    /// *
    Multiply,
    /// /
    Divide,
    /// %
    Modulo,
    /// !
    LogicalNot,
    /// &&
    LogicalAnd,
    /// ||
    LogicalOr,
    /// ~
    BitwiseNot,
    /// ^
    BitwiseXor,
    /// |
    BitwiseOr,
    /// &
    BitwiseAnd,
    /// <<
    ShiftLeft,
    /// >>
    ShiftRight,
    /// ==
    Equal,
    /// !=
    NotEqual,
    /// <
    Less,
    /// >
    Greater,
    /// >=
    GreaterEqual,
    /// <=
    LessEqual,
    /// ->
    Arrow,
    /// .
    Dot,
}

impl<'r, 'a> From<&'r SpanRef<'a>> for TokenData {
    fn from(source: &'r SpanRef<'a>) -> Self {
        match source.content() {
            "(" => Self::OpenParen,
            ")" => Self::CloseParen,
            "{" => Self::OpenBrace,
            "}" => Self::CloseBrace,
            "[" => Self::OpenBracket,
            "]" => Self::CloseBracket,
            ";" => Self::Semicolon,
            "," => Self::Comma,
            "?" => Self::QuestionMark,
            ":" => Self::Colon,
            "#" => Self::Hashtag,

            "=" => Self::Assign(Assignment::Assign),
            "+=" => Self::Assign(Assignment::Add),
            "-=" => Self::Assign(Assignment::Sub),
            "*=" => Self::Assign(Assignment::Multiply),
            "/=" => Self::Assign(Assignment::Divide),
            "%=" => Self::Assign(Assignment::Modulo),
            "|=" => Self::Assign(Assignment::BitwiseOr),
            "&=" => Self::Assign(Assignment::BitwiseAnd),
            "^=" => Self::Assign(Assignment::BitwiseXor),
            "<<=" => Self::Assign(Assignment::ShiftLeft),
            ">>=" => Self::Assign(Assignment::ShiftRight),

            "void" => Self::Keyword(Keyword::DataType(DataType::Void)),
            "short" => Self::Keyword(Keyword::DataType(DataType::Short)),
            "char" => Self::Keyword(Keyword::DataType(DataType::Char)),
            "int" => Self::Keyword(Keyword::DataType(DataType::Int)),
            "long" => Self::Keyword(Keyword::DataType(DataType::Long)),
            "float" => Self::Keyword(Keyword::DataType(DataType::Float)),
            "double" => Self::Keyword(Keyword::DataType(DataType::Double)),
            "enum" => Self::Keyword(Keyword::DataType(DataType::Enum)),
            "struct" => Self::Keyword(Keyword::DataType(DataType::Struct)),
            "union" => Self::Keyword(Keyword::DataType(DataType::Union)),
            "unsigned" => Self::Keyword(Keyword::DataType(DataType::Unsigned)),
            "signed" => Self::Keyword(Keyword::DataType(DataType::Signed)),

            "if" => Self::Keyword(Keyword::ControlFlow(ControlFlow::If)),
            "else" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Else)),
            "switch" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Switch)),
            "case" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Case)),
            "while" => Self::Keyword(Keyword::ControlFlow(ControlFlow::While)),
            "for" => Self::Keyword(Keyword::ControlFlow(ControlFlow::For)),
            "do" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Do)),
            "goto" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Goto)),
            "break" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Break)),
            "continue" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Continue)),
            "return" => Self::Keyword(Keyword::ControlFlow(ControlFlow::Return)),

            "+" => Self::Operator(Operator::Add),
            "++" => Self::Operator(Operator::Increment),
            "-" => Self::Operator(Operator::Sub),
            "--" => Self::Operator(Operator::Decrement),
            "*" => Self::Operator(Operator::Multiply),
            "/" => Self::Operator(Operator::Divide),
            "%" => Self::Operator(Operator::Modulo),

            "!" => Self::Operator(Operator::LogicalNot),
            "||" => Self::Operator(Operator::LogicalOr),
            "&&" => Self::Operator(Operator::LogicalAnd),

            "~" => Self::Operator(Operator::BitwiseNot),
            "&" => Self::Operator(Operator::BitwiseAnd),
            "|" => Self::Operator(Operator::BitwiseOr),
            "^" => Self::Operator(Operator::BitwiseXor),
            "<<" => Self::Operator(Operator::ShiftLeft),
            ">>" => Self::Operator(Operator::ShiftRight),

            "<" => Self::Operator(Operator::Less),
            ">" => Self::Operator(Operator::Greater),
            ">=" => Self::Operator(Operator::GreaterEqual),
            "<=" => Self::Operator(Operator::LessEqual),
            "==" => Self::Operator(Operator::Equal),
            "!=" => Self::Operator(Operator::NotEqual),

            "->" => Self::Operator(Operator::Arrow),
            "." => Self::Operator(Operator::Dot),

            lit => Self::Literal {
                content: lit.to_owned(),
            },
        }
    }
}
