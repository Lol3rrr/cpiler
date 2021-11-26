use general::{Span, SpanData};
use syntax::{Expression, Identifier, SingleOperation};

use crate::{APrimitive, AType, SemanticError, VariableContainer};

#[derive(Debug, PartialEq, Clone)]
pub enum AExpression {
    Literal(Literal),
    Variable {
        ident: Identifier,
        ty: AType,
    },
    FunctionCall {
        name: Identifier,
        arguments: Vec<AExpression>,
        result_ty: AType,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(SpanData<u64>),
    FloatingPoint(SpanData<f64>),
    StringLiteral(SpanData<String>),
}

#[derive(Debug, PartialEq)]
pub enum EvaluationValue {
    Integer(i64),
    FloatingPoint(f64),
}

#[derive(Debug, PartialEq)]
pub enum EvaluationError {
    Pointers,
}

impl AExpression {
    pub fn parse<VC>(raw: Expression, vars: &VC) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            Expression::Literal { content } => {
                if content.data.contains('.') {
                    let value: f64 = content.data.parse().unwrap();

                    let span_d = SpanData {
                        span: content.span,
                        data: value,
                    };
                    Ok(Self::Literal(Literal::FloatingPoint(span_d)))
                } else {
                    let value: u64 = match content.data.parse() {
                        Ok(v) => v,
                        Err(e) => {
                            dbg!(e);
                            panic!("Parsing Integer Type");
                        }
                    };

                    let span_d = SpanData {
                        span: content.span,
                        data: value,
                    };
                    Ok(Self::Literal(Literal::Integer(span_d)))
                }
            }
            Expression::StringLiteral { content } => {
                Ok(Self::Literal(Literal::StringLiteral(content)))
            }
            Expression::Identifier { ident } => {
                dbg!(&ident);

                let (var_type, var_span) = match vars.get_var(&ident) {
                    Some(tmp) => tmp,
                    None => {
                        return Err(SemanticError::UnknownIdentifier { name: ident });
                    }
                };

                dbg!(&var_type, &var_span);

                Ok(AExpression::Variable {
                    ident,
                    ty: var_type.clone(),
                })
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::FuntionCall(raw_args),
            } => {
                let name = match *base {
                    Expression::Identifier { ident } => ident,
                    other => unreachable!("The Function-Call Operation should always only be applied to an identifier: {:?}", other),
                };

                let args = {
                    let mut tmp = Vec::new();
                    for tmp_arg in raw_args {
                        let tmp_res = Self::parse(tmp_arg, vars)?;
                        tmp.push(tmp_res);
                    }

                    tmp
                };
                dbg!(&name, &args);

                let (r_type, arg_types, f_span) = match vars.get_func(&name) {
                    Some(tmp) => tmp,
                    None => return Err(SemanticError::UnknownIdentifier { name }),
                };

                dbg!(&r_type, &arg_types, &f_span);

                if args.len() != arg_types.len() {
                    return Err(SemanticError::MismatchedFunctionArgsCount {
                        expected: SpanData {
                            span: f_span.clone(),
                            data: arg_types.len(),
                        },
                        received: SpanData {
                            span: name.0.span.clone(),
                            data: args.len(),
                        },
                    });
                }

                let mut arg_iter = arg_types.iter().zip(args.iter());
                let found = arg_iter.find(|(exp, recv)| *exp != &recv.result_type());
                if let Some((expected, received)) = found {
                    return Err(SemanticError::MismatchedTypes {
                        expected: SpanData {
                            span: f_span.clone(),
                            data: expected.clone(),
                        },
                        received: SpanData {
                            span: received.entire_span(),
                            data: received.result_type(),
                        },
                    });
                }

                dbg!(&name, &args, &r_type);

                Ok(Self::FunctionCall {
                    name,
                    arguments: args,
                    result_ty: r_type.clone(),
                })
            }
            Expression::SingleOperation { base, operation } => {
                dbg!(&base, &operation);

                todo!("Parse Single Operation")
            }
            unknown => panic!("Unknown Expression: {:?}", unknown),
        }
    }

    pub fn const_evaluate(&self) -> Result<EvaluationValue, EvaluationError> {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(SpanData { data, .. }) => {
                    Ok(EvaluationValue::Integer(*data as i64))
                }
                Literal::FloatingPoint(SpanData { data, .. }) => {
                    Ok(EvaluationValue::FloatingPoint(*data))
                }
                Literal::StringLiteral(SpanData { .. }) => Err(EvaluationError::Pointers),
            },
            unknown => panic!("Dont know how to evaluate {:?}", unknown),
        }
    }

    pub fn result_type(&self) -> AType {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(_) => AType::Primitve(APrimitive::Int),
                Literal::FloatingPoint(_) => AType::Primitve(APrimitive::Float),
                Literal::StringLiteral(_) => {
                    AType::Pointer(Box::new(AType::Primitve(APrimitive::Char)))
                }
            },
            Self::Variable { ty, .. } => ty.clone(),
            Self::FunctionCall { result_ty, .. } => result_ty.clone(),
        }
    }
    pub fn entire_span(&self) -> Span {
        match &self {
            Self::Literal(lit) => match lit {
                Literal::Integer(SpanData { span, .. }) => span.clone(),
                Literal::FloatingPoint(SpanData { span, .. }) => span.clone(),
                Literal::StringLiteral(SpanData { span, .. }) => span.clone(),
            },
            Self::Variable { ident, .. } => ident.0.span.clone(),
            Self::FunctionCall { name, .. } => name.0.span.clone(),
        }
    }
}
