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
    VarArgs,
    QuestionMark,
    Colon,
    Hashtag,
    OpenParen,
    CloseParen,
    /// {
    OpenBrace,
    /// }
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Comment {
        content: String,
    },
    Operator(Operator),
    Literal {
        content: String,
    },
    StringLiteral {
        content: String,
    },
    CharLiteral {
        content: String,
    },
    CompilerDirective {
        content: String,
    },
}

impl Display for TokenData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::Keyword(k) => write!(f, "{}", k),
            Self::Assign(a) => write!(f, "{}", a),
            Self::Semicolon => write!(f, ";"),
            Self::Comma => write!(f, ","),
            Self::VarArgs => write!(f, "..."),
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
            Self::Operator(op) => write!(f, "{}", op),
            Self::Literal { content } => write!(f, "{}", content),
            Self::StringLiteral { content } => write!(f, "\"{}\"", content),
            Self::CharLiteral { content } => write!(f, "'{}'", content),
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

impl Display for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Assign => write!(f, "="),
            Self::Add => write!(f, "+="),
            Self::Sub => write!(f, "-="),
            Self::Multiply => write!(f, "*="),
            Self::Divide => write!(f, "/="),
            Self::Modulo => write!(f, "%="),
            Self::ShiftLeft => write!(f, "<<="),
            Self::ShiftRight => write!(f, ">>="),
            Self::BitwiseAnd => write!(f, "&="),
            Self::BitwiseOr => write!(f, "|="),
            Self::BitwiseXor => write!(f, "^="),
        }
    }
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

impl Display for Keyword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DataType(dt) => write!(f, "{}", dt),
            Self::ControlFlow(cf) => write!(f, "{}", cf),
            Self::Auto => todo!("Format auto"),
            Self::Const => write!(f, "const"),
            Self::Default_ => todo!("Format default_"),
            Self::Bool_ => todo!("Format bool_"),
            Self::Complex_ => todo!("Format complex_"),
            Self::Extern => todo!("Format extern"),
            Self::Imaginary_ => todo!("Format imaginary"),
            Self::Inline => todo!("Format inline"),
            Self::Register => todo!("Format register"),
            Self::Restrict => todo!("Format restrict"),
            Self::SizeOf => write!(f, "sizeof"),
            Self::Static => write!(f, "static"),
            Self::TypeDef => write!(f, "typedef"),
            Self::Volatile => write!(f, "volatile"),
        }
    }
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

impl Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Void => write!(f, "void"),
            Self::Short => write!(f, "short"),
            Self::Char => write!(f, "char"),
            Self::Int => write!(f, "int"),
            Self::Long => write!(f, "long"),
            Self::Float => write!(f, "float"),
            Self::Double => write!(f, "double"),
            Self::Enum => write!(f, "enum"),
            Self::Struct => write!(f, "struct"),
            Self::Union => write!(f, "union"),
            Self::Unsigned => write!(f, "unsigned"),
            Self::Signed => write!(f, "signed"),
        }
    }
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

impl Display for ControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::Switch => write!(f, "switch"),
            Self::Case => write!(f, "case"),
            Self::While => write!(f, "while"),
            Self::For => write!(f, "for"),
            Self::Do => write!(f, "do"),
            Self::Goto => write!(f, "goto"),
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
            Self::Return => write!(f, "return"),
        }
    }
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

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Increment => write!(f, "++"),
            Self::Sub => write!(f, "-"),
            Self::Decrement => write!(f, "--"),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Modulo => write!(f, "%"),
            Self::LogicalNot => write!(f, "!"),
            Self::LogicalAnd => write!(f, "&&"),
            Self::LogicalOr => write!(f, "||"),
            Self::BitwiseNot => write!(f, "~"),
            Self::BitwiseXor => write!(f, "^"),
            Self::BitwiseOr => write!(f, "|"),
            Self::BitwiseAnd => write!(f, "&"),
            Self::ShiftLeft => write!(f, "<<"),
            Self::ShiftRight => write!(f, ">>"),
            Self::Equal => write!(f, "=="),
            Self::NotEqual => write!(f, "!="),
            Self::Less => write!(f, "<"),
            Self::Greater => write!(f, ">"),
            Self::GreaterEqual => write!(f, ">="),
            Self::LessEqual => write!(f, "<="),
            Self::Arrow => write!(f, "->"),
            Self::Dot => write!(f, "."),
        }
    }
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
            "..." => Self::VarArgs,

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
            "const" => Self::Keyword(Keyword::Const),

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

            "typedef" => Self::Keyword(Keyword::TypeDef),

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
