use general::{Span, SpanData};
use ir::{BasicBlock, Constant, Value};
use syntax::{Expression, Identifier, SingleOperation};

use crate::{
    atype, conversion::ConvertContext, APrimitive, AType, SemanticError, TypeDefinitions,
    VariableContainer,
};

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
    SizeOf {
        /// The Type of which we want to calculate the Size
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
    InlineAssembly {
        template: SpanData<String>,
        input_vars: Vec<(Identifier, SpanData<AType>)>,
        output_var: (Identifier, SpanData<AType>),
        span: Span,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Integer(SpanData<i64>),
    FloatingPoint(SpanData<f64>),
    StringLiteral(SpanData<String>),
    CharLiteral(SpanData<char>),
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
    pub fn parse<VC>(
        raw: Expression,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
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
                    let value: i64 = match content.data.parse() {
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
            Expression::CharLiteral { content } => Ok(Self::Literal(Literal::CharLiteral(content))),
            Expression::SizeOf { ty } => {
                let a_ty = AType::parse(ty, ty_defs, vars)?;
                dbg!(&a_ty);

                Ok(Self::SizeOf { ty: a_ty })
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

                let base_exp = AExpression::parse(*base, ty_defs, vars)?;
                dbg!(&base_exp);

                let base_ty = base_exp.result_type();
                dbg!(&base_ty);

                let struct_def = match base_ty.get_struct_def() {
                    Some(s) => s,
                    None => {
                        dbg!(&base_ty);

                        todo!("Wrong Type, expected Struct");
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
                operation: SingleOperation::FuntionCall(mut raw_args),
            } => {
                let name = match *base {
                    Expression::Identifier { ident } => ident,
                    other => unreachable!("The Function-Call Operation should always only be applied to an identifier: {:?}", other),
                };

                if name.0.data == "asm" {
                    dbg!(&raw_args);

                    if raw_args.len() != 3 {
                        panic!("Expected exactly 3 Arguments");
                    }

                    let raw_template_arg = raw_args.remove(0);
                    let template_arg = AExpression::parse(raw_template_arg, ty_defs, vars)?;

                    let template = match template_arg {
                        AExpression::Literal(Literal::StringLiteral(data)) => data,
                        _ => panic!("Expected String Literal"),
                    };
                    dbg!(&template);

                    let raw_input_args = raw_args.remove(0);
                    let input_vars = match raw_input_args {
                        Expression::ArrayLiteral { parts } => {
                            let mut tmp: Vec<(Identifier, SpanData<AType>)> = Vec::new();
                            for part in parts.data {
                                let ident = match part {
                                    Expression::Identifier { ident } => ident,
                                    unexpected => panic!("Expected Identifier: {:?}", unexpected),
                                };

                                let ty_info = match vars.get_var(&ident) {
                                    Some(v) => SpanData {
                                        span: v.1.clone(),
                                        data: v.0.clone(),
                                    },
                                    None => panic!("Unknown Variable: {:?}", &ident),
                                };

                                tmp.push((ident, ty_info));
                            }

                            tmp
                        }
                        _ => panic!("Expected ArrayLiteral of Variables"),
                    };
                    dbg!(&input_vars);

                    let raw_output_arg = raw_args.remove(0);
                    let output_var = match raw_output_arg {
                        Expression::Identifier { ident } => {
                            dbg!(&ident);

                            match vars.get_var(&ident) {
                                Some(v) => (
                                    ident,
                                    SpanData {
                                        span: v.1.clone(),
                                        data: v.0.clone(),
                                    },
                                ),
                                None => panic!("Unknown Variable: {:?}", &ident),
                            }
                        }
                        _ => panic!("Expected Variable Identifier for output"),
                    };
                    dbg!(&output_var);

                    return Ok(Self::InlineAssembly {
                        span: name.0.span,
                        template,
                        output_var,
                        input_vars,
                    });
                }

                let args = {
                    let mut tmp = Vec::new();
                    for tmp_arg in raw_args {
                        let tmp_res = Self::parse(tmp_arg, ty_defs, vars)?;
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
                        atype::assign_type::determine_type(
                            recveived,
                            (&expected.data.ty, &expected.span),
                        )
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
                let base_exp = Self::parse(*base, ty_defs, vars)?;

                let base_ty = base_exp.result_type();

                let elem_ty = match base_ty {
                    AType::Array(arr) => arr.ty,
                    AType::Pointer(inner) => inner,
                    other => {
                        dbg!(&other);

                        todo!("Expected Array");
                    }
                };

                let a_index = AExpression::parse(*index, ty_defs, vars)?;

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

                let a_base = Self::parse(*base, ty_defs, vars)?;
                dbg!(&a_base);

                match a_base {
                    AExpression::Variable { .. } => {}
                    AExpression::ArrayAccess { .. } => {}
                    _ => {
                        todo!("Cant take address of this Expression");
                    }
                };

                let base_ty = a_base.result_type();
                let target_ty = AType::Pointer(Box::new(base_ty));

                Ok(Self::AddressOf {
                    base: Box::new(a_base),
                    ty: target_ty,
                })
            }
            Expression::SingleOperation { base, operation } => {
                dbg!(&base, &operation);

                let a_base = AExpression::parse(*base, ty_defs, vars)?;
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

                let left_a = Self::parse(*left, ty_defs, vars)?;
                dbg!(&left_a);

                let right_a = Self::parse(*right, ty_defs, vars)?;
                dbg!(&right_a);

                let op_a = AOperator::from(operation);
                dbg!(&op_a);

                let (left_exp, right_exp) = match &op_a {
                    AOperator::Comparison(_) => {
                        // TODO
                        // Check for Type Compatibility when comparing
                        (left_a, right_a)
                    }
                    AOperator::Combinator(_) => {
                        // TODO
                        // Check for Type Compatibility when combining logic
                        (left_a, right_a)
                    }
                    AOperator::Arithmetic(_) => atype::determine_types(left_a, right_a)?,
                };

                dbg!(&left_exp, &right_exp);

                Ok(Self::BinaryOperator {
                    left: Box::new(left_exp),
                    right: Box::new(right_exp),
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
                Literal::CharLiteral(SpanData { .. }) => todo!("Allow Char const eval"),
            },
            unknown => panic!("Dont know how to evaluate {:?}", unknown),
        }
    }

    pub fn result_type(&self) -> AType {
        match self {
            Self::Literal(lit) => match lit {
                Literal::Integer(_) => AType::Primitve(APrimitive::LongInt),
                Literal::FloatingPoint(_) => AType::Primitve(APrimitive::Float),
                Literal::StringLiteral(_) => {
                    AType::Pointer(Box::new(AType::Primitve(APrimitive::Char)))
                }
                Literal::CharLiteral(_) => AType::Primitve(APrimitive::Char),
            },
            Self::Variable { ty, .. } => ty.clone(),
            Self::AddressOf { ty, .. } => ty.clone(),
            Self::SizeOf { .. } => AType::Primitve(APrimitive::UnsignedInt),
            Self::ArrayAccess { ty, .. } => ty.clone(),
            Self::StructAccess { ty, .. } => ty.clone(),
            Self::FunctionCall { result_ty, .. } => result_ty.clone(),
            Self::ImplicitCast { target, .. } => target.clone(),
            Self::BinaryOperator { op, left, right } => match op {
                AOperator::Comparison(_) => AType::Primitve(APrimitive::Int),
                AOperator::Combinator(_) => AType::Primitve(APrimitive::Int),
                AOperator::Arithmetic(_) => {
                    debug_assert_eq!(left.result_type(), right.result_type());

                    left.result_type()
                }
            },
            Self::UnaryOperator { op, .. } => match op {
                UnaryOperator::Arithmetic(_) => AType::Primitve(APrimitive::Int),
                UnaryOperator::Logic(_) => AType::Primitve(APrimitive::Int),
            },
            Self::InlineAssembly { .. } => AType::Primitve(APrimitive::Void),
        }
    }
    pub fn entire_span(&self) -> Span {
        match &self {
            Self::Literal(lit) => match lit {
                Literal::Integer(SpanData { span, .. }) => span.clone(),
                Literal::FloatingPoint(SpanData { span, .. }) => span.clone(),
                Literal::StringLiteral(SpanData { span, .. }) => span.clone(),
                Literal::CharLiteral(SpanData { span, .. }) => span.clone(),
            },
            Self::Variable { ident, .. } => ident.0.span.clone(),
            Self::AddressOf { base, .. } => base.entire_span(),
            Self::SizeOf { .. } => panic!("SizeOf Operand"),
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
            Self::InlineAssembly { span, .. } => span.clone(),
        }
    }

    fn val_to_operand(value: Value, block: &BasicBlock, ctx: &ConvertContext) -> ir::Operand {
        match value {
            Value::Unknown => {
                todo!("Unknown Value as Operand")
            }
            Value::Phi { sources } => {
                dbg!(&sources);

                todo!("Phi as Operand")
            }
            Value::Constant(constant) => ir::Operand::Constant(constant),
            Value::Variable(var) => ir::Operand::Variable(var),
            Value::Expression(exp) => {
                dbg!(&exp);

                match &exp {
                    ir::Expression::Cast { base, target } => {
                        dbg!(&base, &target);

                        let tmp_var = ir::Variable::tmp(ctx.next_tmp(), target.clone());
                        dbg!(&tmp_var);

                        let assign_statement = ir::Statement::Assignment {
                            target: tmp_var.clone(),
                            value: Value::Expression(exp),
                        };
                        block.add_statement(assign_statement);

                        ir::Operand::Variable(tmp_var)
                    }
                    ir::Expression::BinaryOp { op, left, right } => {
                        dbg!(&op, &left, &right);

                        let tmp_var = ir::Variable::tmp(ctx.next_tmp(), left.ty());
                        dbg!(&tmp_var);

                        let assign_statement = ir::Statement::Assignment {
                            target: tmp_var.clone(),
                            value: Value::Expression(exp),
                        };
                        block.add_statement(assign_statement);

                        ir::Operand::Variable(tmp_var)
                    }
                    other => panic!("{:?} as Operand", other),
                }
            }
        }
    }

    /// Converts the Expression to the corresponding IR
    pub fn to_ir(self, block: &BasicBlock, ctx: &ConvertContext) -> Value {
        match self {
            AExpression::Literal(lit) => match lit {
                Literal::Integer(SpanData { data, .. }) => Value::Constant(Constant::I64(data)),
                other => {
                    dbg!(&other);

                    todo!("Convert Literal");
                }
            },
            AExpression::Variable { ident, ty } => {
                dbg!(&ident, &ty);

                let var = block.definition(&ident.0.data, &|| ctx.next_tmp()).unwrap();
                Value::Variable(var)
            }
            AExpression::BinaryOperator { op, left, right } => {
                dbg!(&op, &left, &right);

                let ir_op = op.to_ir();
                dbg!(&ir_op);

                let left_value = left.to_ir(block, ctx);
                dbg!(&left_value);
                let left_operand = Self::val_to_operand(left_value, block, ctx);
                dbg!(&left_operand);

                let right_value = right.to_ir(block, ctx);
                dbg!(&right_value);
                let right_operand = Self::val_to_operand(right_value, block, ctx);
                dbg!(&right_operand);

                Value::Expression(ir::Expression::BinaryOp {
                    op: ir_op,
                    left: left_operand,
                    right: right_operand,
                })
            }
            AExpression::ImplicitCast { base, target } => {
                dbg!(&base, &target);

                let target_ty = target.to_ir();
                dbg!(&target_ty);

                let value = base.to_ir(block, ctx);
                dbg!(&value);
                let val_operand = Self::val_to_operand(value, block, ctx);
                dbg!(&val_operand);

                Value::Expression(ir::Expression::Cast {
                    target: target_ty,
                    base: val_operand,
                })
            }
            AExpression::UnaryOperator { base, op } => {
                dbg!(&base, &op);

                let base_value = base.to_ir(block, ctx);
                let base_operand = Self::val_to_operand(base_value, block, ctx);
                dbg!(&base_operand);

                match op {
                    UnaryOperator::Arithmetic(UnaryArithmeticOp::SuffixDecrement) => {
                        let base_var = match base_operand {
                            ir::Operand::Variable(v) => v,
                            other => {
                                dbg!(&other);

                                panic!("Suffix-Decrement only applyable for Variables");
                            }
                        };

                        let result_val = Value::Variable(base_var.clone());
                        dbg!(&result_val);

                        let target_var = base_var.next_gen();
                        let update_statement = ir::Statement::Assignment {
                            target: target_var,
                            value: ir::Value::Expression(ir::Expression::UnaryOp {
                                op: ir::UnaryOp::Arith(ir::UnaryArithmeticOp::Decrement),
                                base: ir::Operand::Variable(base_var),
                            }),
                        };
                        block.add_statement(update_statement);

                        result_val
                    }
                    other => {
                        dbg!(&other);

                        todo!("Handle UnaryOp");
                    }
                }
            }
            AExpression::FunctionCall {
                name,
                arguments,
                result_ty,
            } => {
                dbg!(&name, &arguments, &result_ty);

                let name = name.0.data;

                let args = Vec::new();
                for tmp_arg in arguments {
                    dbg!(&tmp_arg);

                    todo!("Handle Arguments");
                }
                let ty = result_ty.to_ir();

                let tmp_var = ir::Variable::tmp(ctx.next_tmp(), ty.clone());

                let func_statement = ir::Statement::Assignment {
                    target: tmp_var.clone(),
                    value: Value::Expression(ir::Expression::FunctionCall {
                        name,
                        arguments: args,
                        return_ty: ty,
                    }),
                };
                block.add_statement(func_statement);

                Value::Variable(tmp_var)
            }
            other => {
                dbg!(&other);

                todo!("Convert Expression");
            }
        }
    }
}
