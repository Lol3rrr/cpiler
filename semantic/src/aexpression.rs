use general::{Span, SpanData};
use syntax::{Expression, Identifier, SingleOperation};

use crate::{APrimitive, AType, SemanticError, VariableContainer};

mod operator;
pub use operator::*;

mod unary_operator;
pub use unary_operator::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AExpression {
    Literal(Literal),
    Variable {
        ident: Identifier,
        ty: AType,
    },
    AddressOf {
        base: Box<Self>,
        ty: AType,
    },
    ArrayAccess {
        base: Box<Self>,
        index: Box<Self>,
        ty: AType,
    },
    StructAccess {
        base: Box<Self>,
        field: Identifier,
        ty: AType,
    },
    FunctionCall {
        name: Identifier,
        arguments: Vec<AExpression>,
        result_ty: AType,
    },
    ImplicitCast {
        base: Box<Self>,
        target: AType,
    },
    BinaryOperator {
        left: Box<Self>,
        right: Box<Self>,
        op: AOperator,
    },
    UnaryOperator {
        base: Box<Self>,
        op: UnaryOperator,
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
            Expression::StructAccess { base, field } => {
                dbg!(&base, &field);

                let base_exp = AExpression::parse(*base, vars)?;
                dbg!(&base_exp);

                let base_ty = base_exp.result_type();
                dbg!(&base_ty);

                let struct_def = match base_ty {
                    AType::AnonStruct(def) => def,
                    other => {
                        dbg!(&other);

                        todo!("Expected Struct");
                    }
                };
                dbg!(&struct_def);

                let (field_ty, _) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        dbg!(&field);

                        todo!("Unknown Field on Struct");
                    }
                };
                dbg!(&field_ty);

                Ok(Self::StructAccess {
                    base: Box::new(base_exp),
                    field,
                    ty: field_ty,
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

                let func_dec = match vars.get_func(&name) {
                    Some(tmp) => tmp,
                    None => return Err(SemanticError::UnknownIdentifier { name }),
                };
                dbg!(&func_dec);

                if (args.len() != func_dec.arguments.len() && !func_dec.var_args)
                    || (func_dec.var_args && args.len() < func_dec.arguments.len())
                {
                    return Err(SemanticError::MismatchedFunctionArgsCount {
                        expected: SpanData {
                            span: func_dec.declaration.clone(),
                            data: func_dec.arguments.len(),
                        },
                        received: SpanData {
                            span: name.0.span.clone(),
                            data: args.len(),
                        },
                    });
                }

                let arg_iter = func_dec.arguments.iter().zip(args.into_iter());
                let arg_results: Vec<_> = arg_iter
                    .map(|(expected, recveived)| {
                        let recveived_type = recveived.result_type();
                        if &expected.data.ty == &recveived_type {
                            return Ok(recveived);
                        }

                        if recveived_type.implicitly_castable(&expected.data.ty) {
                            let target = expected.data.ty.clone();
                            let n_exp = Self::ImplicitCast {
                                base: Box::new(recveived),
                                target,
                            };

                            return Ok(n_exp);
                        }

                        Err(SemanticError::MismatchedTypes {
                            expected: SpanData {
                                span: expected.span.clone(),
                                data: expected.data.ty.clone(),
                            },
                            received: SpanData {
                                span: recveived.entire_span(),
                                data: recveived_type,
                            },
                        })
                    })
                    .collect();

                if let Some(err) = arg_results.iter().find(|t| t.is_err()) {
                    return err.clone();
                }

                let args: Vec<_> = arg_results.into_iter().filter_map(|t| t.ok()).collect();
                dbg!(&name, &args, &func_dec.return_ty);

                Ok(Self::FunctionCall {
                    name,
                    arguments: args,
                    result_ty: func_dec.return_ty.clone(),
                })
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::ArrayAccess(index),
            } => {
                let base_exp = Self::parse(*base, vars)?;

                let base_ty = base_exp.result_type();

                let elem_ty = match base_ty {
                    AType::Array(arr) => arr.ty,
                    AType::Pointer(inner) => inner,
                    other => {
                        dbg!(&other);

                        todo!("Expected Array");
                    }
                };

                let a_index = AExpression::parse(*index, vars)?;

                Ok(Self::ArrayAccess {
                    base: Box::new(base_exp),
                    index: Box::new(a_index),
                    ty: *elem_ty,
                })
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::AddressOf,
            } => {
                dbg!(&base);

                let a_base = Self::parse(*base, vars)?;
                dbg!(&a_base);

                match a_base {
                    AExpression::Variable { .. } => {}
                    _ => {
                        todo!("Cant take address of this Expression");
                    }
                };

                let base_ty = a_base.result_type();
                let target_ty = AType::Pointer(Box::new(base_ty.clone()));

                Ok(Self::AddressOf {
                    base: Box::new(a_base),
                    ty: target_ty,
                })
            }
            Expression::SingleOperation { base, operation } => {
                dbg!(&base, &operation);

                let a_base = AExpression::parse(*base, vars)?;
                dbg!(&a_base);

                let a_op = UnaryOperator::from(operation);
                dbg!(&a_op);

                Ok(Self::UnaryOperator {
                    base: Box::new(a_base),
                    op: a_op,
                })
            }
            Expression::Operation {
                left,
                right,
                operation,
            } => {
                dbg!(&left, &right, &operation);

                let left_a = Self::parse(*left, vars)?;
                dbg!(&left_a);

                let right_a = Self::parse(*right, vars)?;
                dbg!(&right_a);

                let op_a = AOperator::from(operation);
                dbg!(&op_a);

                match &op_a {
                    AOperator::Comparison(_) => {
                        // TODO
                        // Check for Type Compatibility when comparing
                    }
                    AOperator::Combinator(_) => {
                        // TODO
                        // Check for Type Compatibility when combining logic
                    }
                    AOperator::Arithmetic(_) => {
                        // TODO
                        // Check for Type Compatibility when performing arithemtic on the ops
                    }
                };

                Ok(Self::BinaryOperator {
                    left: Box::new(left_a),
                    right: Box::new(right_a),
                    op: op_a,
                })
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
            Self::AddressOf { ty, .. } => ty.clone(),
            Self::ArrayAccess { ty, .. } => ty.clone(),
            Self::StructAccess { ty, .. } => ty.clone(),
            Self::FunctionCall { result_ty, .. } => result_ty.clone(),
            Self::ImplicitCast { target, .. } => target.clone(),
            Self::BinaryOperator { op, left, right } => match op {
                AOperator::Comparison(_) => AType::Primitve(APrimitive::Int),
                AOperator::Combinator(_) => AType::Primitve(APrimitive::Int),
                AOperator::Arithmetic(_) => {
                    dbg!(&left, &right);

                    todo!("Determine result type of Arithemtic Operand")
                }
            },
            Self::UnaryOperator { op, .. } => match op {
                UnaryOperator::Arithmetic(_) => AType::Primitve(APrimitive::Int),
            },
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
            Self::AddressOf { base, .. } => base.entire_span(),
            Self::ArrayAccess { base, .. } => base.entire_span(),
            Self::StructAccess { field, .. } => field.0.span.clone(),
            Self::FunctionCall { name, .. } => name.0.span.clone(),
            Self::ImplicitCast { base, .. } => base.entire_span(),
            Self::BinaryOperator { left, right, .. } => {
                let left_span = left.entire_span();
                let right_span = right.entire_span();

                let start = left_span.source_area().start;
                let end = right_span.source_area().end;

                let source = left_span.source().clone();

                Span::new_arc_source(source, start..end)
            }
            Self::UnaryOperator { base, .. } => base.entire_span(),
        }
    }
}
