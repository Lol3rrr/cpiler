use std::collections::BTreeSet;

use general::{Span, SpanData};
use ir::{BasicBlock, Value};
use syntax::{Expression, Identifier, SingleOperation};

use crate::{
    atype, conversion::ConvertContext, AAssignTarget, APrimitive, AType, ArrayAccessTarget,
    InvalidOperation, SemanticError, StructDef, StructMember, TypeDefinitions, VariableContainer,
};

mod operator;
pub use operator::*;

mod unary_operator;
pub use unary_operator::*;

mod literal;
pub use literal::*;

mod functioncall;
pub use functioncall::*;

mod structaccess;
pub use structaccess::*;

#[derive(Debug, PartialEq, Clone)]
pub enum AExpression {
    Literal(Literal),
    Variable {
        name: String,
        src: Identifier,
        ty: SpanData<AType>,
    },
    AddressOf {
        base: Box<Self>,
        ty: AType,
    },
    SizeOf {
        /// The Type of which we want to calculate the Size
        ty: AType,
        area: Span,
    },
    ArrayAccess {
        base: Box<Self>,
        index: Box<Self>,
        ty: SpanData<AType>,
    },
    StructAccess(StructAccess),
    FunctionCall(FunctionCall),
    Cast {
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
    InlineConditional {
        condition: Box<Self>,
        left: Box<Self>,
        right: Box<Self>,
    },
    InlineAssembly {
        template: SpanData<String>,
        input_vars: Vec<(Identifier, SpanData<AType>)>,
        output_var: Option<(Identifier, SpanData<AType>)>,
        span: Span,
    },
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
                    let value: f64 = match content.data.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(SemanticError::MismatchedTypes {
                                expected: SpanData {
                                    span: content.span.clone(),
                                    data: AType::Primitve(APrimitive::Double),
                                },
                                received: SpanData {
                                    span: content.span.clone(),
                                    data: AType::Primitve(APrimitive::Void),
                                },
                            });
                        }
                    };

                    let span_d = SpanData {
                        span: content.span,
                        data: value,
                    };
                    Ok(Self::Literal(Literal::FloatingPoint(span_d)))
                } else {
                    let value: i64 = match content.data.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            return Err(SemanticError::MismatchedTypes {
                                expected: SpanData {
                                    span: content.span.clone(),
                                    data: AType::Primitve(APrimitive::LongInt),
                                },
                                received: SpanData {
                                    span: content.span.clone(),
                                    data: AType::Primitve(APrimitive::Void),
                                },
                            });
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
            Expression::SizeOf { ty, area } => {
                let a_ty = AType::parse(ty, ty_defs, vars)?;

                Ok(Self::SizeOf { ty: a_ty, area })
            }
            Expression::Identifier { ident } => {
                let var_dec = match vars.get_var(&ident) {
                    Some(tmp) => tmp,
                    None => {
                        return Err(SemanticError::UnknownIdentifier { name: ident });
                    }
                };

                let var_name = &var_dec.internal_name;
                let var_type = &var_dec.ty;
                let var_span = &var_dec.declaration;

                Ok(AExpression::Variable {
                    name: var_name.clone(),
                    src: ident,
                    ty: SpanData {
                        data: var_type.clone(),
                        span: var_span.clone(),
                    },
                })
            }
            Expression::StructAccess { base, field } => {
                let base_exp = AExpression::parse(*base, ty_defs, vars)?;

                let base_ty = base_exp.result_type();

                let (struct_def, def_span) = match base_ty.get_struct_def() {
                    Some(s) => s,
                    None => {
                        return Err(SemanticError::MismatchedTypes {
                            expected: SpanData {
                                span: field.0.span.clone(),
                                data: atype::AType::Struct {
                                    def: StructDef {
                                        members: vec![StructMember {
                                            name: field.clone(),
                                            ty: AType::Primitve(APrimitive::Void),
                                        }],
                                    },
                                    area: field.0.span,
                                },
                            },
                            received: SpanData {
                                span: base_exp.entire_span(),
                                data: base_ty,
                            },
                        });
                    }
                };

                let (field_ty, _) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        return Err(SemanticError::UnknownStructField {
                            field_name: field,
                            struct_def: SpanData {
                                span: def_span.clone(),
                                data: struct_def.clone(),
                            },
                        });
                    }
                };

                Ok(Self::StructAccess(StructAccess {
                    base: Box::new(base_exp),
                    field,
                    ty: field_ty,
                }))
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::FuntionCall(mut raw_args),
            } => {
                let name = match *base {
                    Expression::Identifier { ident } => ident,
                    other => {
                        return Err(SemanticError::InvalidOperation {
                            base: other.entire_span().unwrap(),
                            operation: InvalidOperation::FunctionCall,
                        })
                    }
                };

                if name.0.data == "asm" {
                    if raw_args.is_empty() {
                        panic!()
                    }

                    let raw_template_arg = raw_args.remove(0);
                    let template_arg = AExpression::parse(raw_template_arg, ty_defs, vars)?;

                    let template = match template_arg {
                        AExpression::Literal(Literal::StringLiteral(data)) => data,
                        _ => panic!("Expected String Literal"),
                    };

                    if raw_args.is_empty() {
                        return Ok(Self::InlineAssembly {
                            span: name.0.span,
                            template,
                            output_var: None,
                            input_vars: Vec::new(),
                        });
                    }

                    let raw_output_arg = raw_args.remove(0);
                    let output_var = match raw_output_arg {
                        Expression::Identifier { ident } => match vars.get_var(&ident) {
                            Some(v) => (
                                ident,
                                SpanData {
                                    span: v.declaration.clone(),
                                    data: v.ty.clone(),
                                },
                            ),
                            None => panic!("Unknown Variable: {:?}", &ident),
                        },
                        _ => panic!("Expected Variable Identifier for output"),
                    };

                    if raw_args.is_empty() {
                        return Ok(Self::InlineAssembly {
                            span: name.0.span,
                            template,
                            output_var: Some(output_var),
                            input_vars: Vec::new(),
                        });
                    }

                    let input_vars: Vec<_> = raw_args
                        .into_iter()
                        .map(|exp| {
                            let ident = match exp {
                                Expression::Identifier { ident } => ident,
                                other => {
                                    dbg!(&other);
                                    panic!("Expected Identifier")
                                }
                            };

                            let ty_info = match vars.get_var(&ident) {
                                Some(v) => SpanData {
                                    span: v.declaration.clone(),
                                    data: v.ty.clone(),
                                },
                                None => panic!("Unknown Variable: {:?}", ident),
                            };

                            (ident, ty_info)
                        })
                        .collect();

                    return Ok(Self::InlineAssembly {
                        span: name.0.span,
                        template,
                        output_var: Some(output_var),
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

                let func_dec = match vars.get_func(&name) {
                    Some(tmp) => tmp,
                    None => return Err(SemanticError::UnknownIdentifier { name }),
                };

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

                Ok(Self::FunctionCall(FunctionCall {
                    name,
                    arguments: args,
                    result_ty: func_dec.return_ty.clone(),
                }))
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
                    _ => {
                        return Err(SemanticError::InvalidOperation {
                            base: base_exp.entire_span(),
                            operation: InvalidOperation::ArrayAccess,
                        })
                    }
                };
                let ty_span = base_exp.entire_span();

                let a_index = AExpression::parse(*index, ty_defs, vars)?;

                Ok(Self::ArrayAccess {
                    base: Box::new(base_exp),
                    index: Box::new(a_index),
                    ty: SpanData {
                        span: ty_span,
                        data: *elem_ty,
                    },
                })
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::AddressOf,
            } => {
                let a_base = Self::parse(*base, ty_defs, vars)?;

                match a_base {
                    AExpression::Variable { .. } => {}
                    AExpression::ArrayAccess { .. } => {}
                    _ => {
                        return Err(SemanticError::InvalidOperation {
                            base: a_base.entire_span(),
                            operation: InvalidOperation::AdressOf,
                        });
                    }
                };

                let base_ty = a_base.result_type();
                let target_ty = AType::Pointer(Box::new(base_ty));

                Ok(Self::AddressOf {
                    base: Box::new(a_base),
                    ty: target_ty,
                })
            }
            Expression::SingleOperation {
                base,
                operation: SingleOperation::Dereference,
            } => {
                let a_base = Self::parse(*base, ty_defs, vars)?;

                let base_ty = a_base.result_type();
                match base_ty {
                    AType::Pointer(_) => {}
                    _ => {
                        return Err(SemanticError::InvalidOperation {
                            base: a_base.entire_span(),
                            operation: InvalidOperation::Dereference,
                        })
                    }
                };

                Ok(Self::UnaryOperator {
                    base: Box::new(a_base),
                    op: UnaryOperator::Derference,
                })
            }
            Expression::SingleOperation { base, operation } => {
                let a_base = AExpression::parse(*base, ty_defs, vars)?;

                let a_op = UnaryOperator::from(operation);

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
                let left_a = Self::parse(*left, ty_defs, vars)?;

                let right_a = Self::parse(*right, ty_defs, vars)?;

                let op_a = AOperator::from(operation);

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
                    AOperator::Bitwise(_) => {
                        // TODO
                        // Check for Type Compatibility when performing bitwise operations
                        (left_a, right_a)
                    }
                };

                Ok(Self::BinaryOperator {
                    left: Box::new(left_exp),
                    right: Box::new(right_exp),
                    op: op_a,
                })
            }
            Expression::Cast { target_ty, exp } => {
                let ty = AType::parse(target_ty, ty_defs, vars)?;

                let inner_exp = Self::parse(*exp, ty_defs, vars)?;

                Ok(Self::Cast {
                    target: ty,
                    base: Box::new(inner_exp),
                })
            }
            Expression::ArrayLiteral { .. } => {
                // dbg!(&parts);

                Err(SemanticError::NotImplemented {
                    ctx: "Array Literals".to_string(),
                })
            }
            Expression::Conditional {
                condition,
                first,
                second,
            } => {
                let condition_exp = Self::parse(*condition, ty_defs, vars)?;
                let left_exp = Self::parse(*first, ty_defs, vars)?;
                let right_exp = Self::parse(*second, ty_defs, vars)?;

                let left_ty = left_exp.result_type();
                let right_ty = right_exp.result_type();
                if left_ty != right_ty {
                    return Err(SemanticError::MismatchedTypes {
                        expected: SpanData {
                            span: left_exp.entire_span(),
                            data: left_ty,
                        },
                        received: SpanData {
                            span: right_exp.entire_span(),
                            data: right_ty,
                        },
                    });
                }

                Ok(Self::InlineConditional {
                    condition: Box::new(condition_exp),
                    left: Box::new(left_exp),
                    right: Box::new(right_exp),
                })
            }
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
            Self::Variable { ty, .. } => ty.data.clone(),
            Self::AddressOf { ty, .. } => ty.clone(),
            Self::SizeOf { .. } => AType::Primitve(APrimitive::UnsignedInt),
            Self::ArrayAccess { ty, .. } => ty.data.clone(),
            Self::StructAccess(StructAccess { ty, .. }) => ty.clone(),
            Self::FunctionCall(FunctionCall { result_ty, .. }) => result_ty.clone(),
            Self::Cast { target, .. } => target.clone(),
            Self::BinaryOperator { op, left, right } => match op {
                AOperator::Comparison(_) => AType::Primitve(APrimitive::Int),
                AOperator::Combinator(_) => AType::Primitve(APrimitive::Int),
                AOperator::Arithmetic(_) => {
                    debug_assert_eq!(left.result_type(), right.result_type());

                    left.result_type()
                }
                AOperator::Bitwise(_) => AType::Primitve(APrimitive::Int),
            },
            Self::UnaryOperator { op, base } => match op {
                UnaryOperator::Arithmetic(_) => AType::Primitve(APrimitive::Int),
                UnaryOperator::Logic(_) => AType::Primitve(APrimitive::Int),
                UnaryOperator::Bitwise(_) => AType::Primitve(APrimitive::Int),
                UnaryOperator::Derference => match base.result_type() {
                    AType::Pointer(inner) => *inner,
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                },
            },
            Self::InlineConditional { left, right, .. } => {
                debug_assert_eq!(left.result_type(), right.result_type());

                left.result_type()
            }
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
            Self::Variable { src, .. } => src.0.span.clone(),
            Self::AddressOf { base, .. } => base.entire_span(),
            Self::SizeOf { area, .. } => area.clone(),
            Self::ArrayAccess { base, .. } => base.entire_span(),
            Self::StructAccess(StructAccess { field, .. }) => field.0.span.clone(),
            Self::FunctionCall(FunctionCall { name, .. }) => name.0.span.clone(),
            Self::Cast { base, .. } => base.entire_span(),
            Self::BinaryOperator { left, right, .. } => {
                let left_span = left.entire_span();
                let right_span = right.entire_span();

                let start = left_span.source_area().start;
                let end = right_span.source_area().end;

                let source = left_span.source().clone();

                Span::new_arc_source(source, start..end)
            }
            Self::UnaryOperator { base, .. } => base.entire_span(),
            Self::InlineConditional {
                condition, right, ..
            } => {
                let conditional_span = condition.entire_span();
                let right_span = right.entire_span();

                conditional_span.join(right_span)
            }
            Self::InlineAssembly { span, .. } => span.clone(),
        }
    }

    pub fn assign_target(self) -> AAssignTarget {
        match self {
            Self::Variable { name, src, ty } => AAssignTarget::Variable {
                name,
                src,
                ty_info: ty,
            },
            Self::ArrayAccess { base, index, ty } => {
                AAssignTarget::ArrayAccess(ArrayAccessTarget {
                    target: Box::new(base.assign_target()),
                    index,
                    ty_info: ty,
                })
            }
            other => {
                dbg!(&other);

                todo!()
            }
        }
    }

    /// Returns a Set of Variables that are used in this Condition
    pub fn used_variables(&self) -> BTreeSet<String> {
        match self {
            Self::Literal(_) => BTreeSet::new(),
            Self::Variable { name, .. } => {
                let mut tmp = BTreeSet::new();
                tmp.insert(name.clone());
                tmp
            }
            Self::AddressOf { base, .. } => base.used_variables(),
            Self::SizeOf { .. } => BTreeSet::new(),
            Self::ArrayAccess { base, index, .. } => {
                let mut tmp = BTreeSet::new();

                tmp.extend(base.used_variables());
                tmp.extend(index.used_variables());

                tmp
            }
            Self::StructAccess(StructAccess { base, .. }) => base.used_variables(),
            Self::FunctionCall(call) => {
                let mut tmp = BTreeSet::new();

                for arg in call.arguments.iter() {
                    tmp.extend(arg.used_variables());
                }

                tmp
            }
            Self::Cast { base, .. } => base.used_variables(),
            Self::BinaryOperator { left, right, .. } => {
                let mut tmp = BTreeSet::new();
                tmp.extend(left.used_variables());
                tmp.extend(right.used_variables());
                tmp
            }
            Self::UnaryOperator { base, .. } => base.used_variables(),
            Self::InlineConditional {
                condition,
                left,
                right,
            } => {
                let mut tmp = BTreeSet::new();
                tmp.extend(condition.used_variables());
                tmp.extend(left.used_variables());
                tmp.extend(right.used_variables());
                tmp
            }
            Self::InlineAssembly {
                output_var,
                input_vars,
                ..
            } => {
                println!("Inputs: {:?}", input_vars);
                println!("Output: {:?}", output_var);
                todo!();
            }
        }
    }

    pub fn val_to_operand(value: Value, block: &BasicBlock, ctx: &ConvertContext) -> ir::Operand {
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
            Value::Expression(exp) => match &exp {
                ir::Expression::Cast { target, .. } => {
                    let tmp_var = ir::Variable::tmp(ctx.next_tmp(), target.clone())
                        .set_description("Temp Variable for Cast");

                    let assign_statement = ir::Statement::Assignment {
                        target: tmp_var.clone(),
                        value: Value::Expression(exp),
                    };
                    block.add_statement(assign_statement);

                    ir::Operand::Variable(tmp_var)
                }
                ir::Expression::ReadMemory { read_ty, .. } => {
                    let tmp_var = ir::Variable::tmp(ctx.next_tmp(), read_ty.clone())
                        .set_description("Memory Address for Read Memory Expression");

                    let assign_statement = ir::Statement::Assignment {
                        target: tmp_var.clone(),
                        value: Value::Expression(exp),
                    };
                    block.add_statement(assign_statement);

                    ir::Operand::Variable(tmp_var)
                }
                ir::Expression::BinaryOp { left, .. } => {
                    let tmp_var = ir::Variable::tmp(ctx.next_tmp(), left.ty())
                        .set_description("Temp Variable for Binary Operation");

                    let assign_statement = ir::Statement::Assignment {
                        target: tmp_var.clone(),
                        value: Value::Expression(exp),
                    };
                    block.add_statement(assign_statement);

                    ir::Operand::Variable(tmp_var)
                }
                ir::Expression::UnaryOp { base, .. } => {
                    let tmp_var = ir::Variable::tmp(ctx.next_tmp(), base.ty())
                        .set_description("Temp Variable for Unary Operation");

                    let assign_statement = ir::Statement::Assignment {
                        target: tmp_var.clone(),
                        value: Value::Expression(exp),
                    };
                    block.add_statement(assign_statement);

                    ir::Operand::Variable(tmp_var)
                }
                other => panic!("{:?} as Operand", other),
            },
        }
    }

    /// Converts the Expression to the corresponding IR
    pub fn to_ir(self, block: &mut BasicBlock, ctx: &ConvertContext) -> Value {
        match self {
            AExpression::Literal(lit) => lit.to_value(block, ctx),
            AExpression::Variable { name, .. } => {
                let var = block.definition(&name, &|| ctx.next_tmp()).unwrap();
                if var.global() {
                    let next_var = var.next_gen();
                    block.add_statement(ir::Statement::Assignment {
                        target: next_var.clone(),
                        value: ir::Value::Expression(ir::Expression::ReadGlobalVariable {
                            name: var.name,
                        }),
                    });

                    return Value::Variable(next_var);
                }

                Value::Variable(var)
            }
            AExpression::BinaryOperator { op, left, right } => {
                let ir_op = op.to_ir();

                let left_value = left.to_ir(block, ctx);
                let left_operand = Self::val_to_operand(left_value, block, ctx);

                let right_value = right.to_ir(block, ctx);
                let right_operand = Self::val_to_operand(right_value, block, ctx);

                Value::Expression(ir::Expression::BinaryOp {
                    op: ir_op,
                    left: left_operand,
                    right: right_operand,
                })
            }
            AExpression::Cast { base, target } => {
                let target_ty = target.to_ir();

                let value = base.to_ir(block, ctx);
                let val_operand = Self::val_to_operand(value, block, ctx);

                Value::Expression(ir::Expression::Cast {
                    target: target_ty,
                    base: val_operand,
                })
            }
            AExpression::UnaryOperator { base, op } => op.to_ir(base, block, ctx),
            AExpression::FunctionCall(call) => call.to_ir(block, ctx),
            AExpression::AddressOf { base, .. } => {
                let base_value = base.ir_address(block, ctx);

                match &base_value {
                    ir::Value::Variable(_) => {
                        let base_oper = Self::val_to_operand(base_value, block, ctx);

                        Value::Expression(ir::Expression::AdressOf { base: base_oper })
                    }
                    ir::Value::Expression(_) => base_value,
                    other => {
                        dbg!(&other);
                        todo!()
                    }
                }
            }
            AExpression::ArrayAccess { base, ty, index } => {
                let base_address_value = base.ir_address(block, ctx);
                let base_oper = Self::val_to_operand(base_address_value, block, ctx);

                let index_value = index.to_ir(block, ctx);
                let index_oper = Self::val_to_operand(index_value, block, ctx);

                let element_size = ty.data.byte_size(ctx.arch());

                let offset_value = ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Multiply),
                    left: index_oper,
                    right: ir::Operand::Constant(ir::Constant::I64(element_size as i64)),
                });
                let offset_oper = Self::val_to_operand(offset_value, block, ctx);

                let target_addr_exp = ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: base_oper,
                    right: offset_oper,
                };

                let target_ty = ty.data.ty();
                match &target_ty {
                    AType::Primitve(_) => {
                        let target_addr_oper = Self::val_to_operand(
                            ir::Value::Expression(target_addr_exp),
                            block,
                            ctx,
                        );

                        Value::Expression(ir::Expression::ReadMemory {
                            address: target_addr_oper,
                            read_ty: target_ty.to_ir(),
                        })
                    }
                    AType::Array(_) | AType::Struct { .. } => {
                        ir::Value::Expression(target_addr_exp)
                    }
                    other => {
                        dbg!(&other);

                        todo!("Array Access for non Primitive Type");
                    }
                }
            }
            AExpression::StructAccess(StructAccess { base, field, .. }) => {
                let base_ty = base.result_type().ty();
                let (s_def, _) = base_ty.get_struct_def().unwrap();

                let base_addr_value = base.ir_address(block, ctx);
                let base_oper = Self::val_to_operand(base_addr_value, block, ctx);

                let raw_field_ty = s_def.find_member(&field).unwrap().data;
                let field_ty = raw_field_ty.to_ir();

                let offset = s_def.member_offset(&field.0.data, ctx.arch()).unwrap();

                let offset_value = Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: base_oper,
                    right: ir::Operand::Constant(ir::Constant::I64(offset as i64)),
                });

                let offset_oper = Self::val_to_operand(offset_value, block, ctx);

                Value::Expression(ir::Expression::ReadMemory {
                    address: offset_oper,
                    read_ty: field_ty,
                })
            }
            AExpression::SizeOf { ty, .. } => {
                let size = ty.byte_size(ctx.arch());

                ir::Value::Constant(ir::Constant::I64(size as i64))
            }
            other => {
                dbg!(&other);

                todo!("Convert Expression");
            }
        }
    }

    /// This is used to convert the Expression into a Value that contains the Target Address for
    /// some other access.
    /// The resulting address will then be used in things like "ReadMemory" or "WriteMemory" or to
    /// calculate some offest from it, like for a struct or array access
    pub fn ir_address(self, block: &mut BasicBlock, ctx: &ConvertContext) -> ir::Value {
        match self {
            Self::Variable { name, ty, .. } => {
                match ty.data.ty() {
                    AType::Pointer(_) | AType::Array(_) | AType::Struct { .. } => {}
                    other => {
                        dbg!(&other);

                        todo!("Unexpected Variable")
                    }
                };
                let var = block.definition(&name, &|| ctx.next_tmp()).unwrap();

                if var.global() {
                    let next_var = var.next_gen();
                    block.add_statement(ir::Statement::Assignment {
                        target: next_var.clone(),
                        value: ir::Value::Expression(ir::Expression::ReadGlobalVariable {
                            name: var.name,
                        }),
                    });

                    return Value::Variable(next_var);
                }

                ir::Value::Variable(var)
            }
            Self::ArrayAccess { base, ty, index } => {
                let base_address = base.ir_address(block, ctx);

                let index_value = index.to_ir(block, ctx);
                let index_oper = Self::val_to_operand(index_value, block, ctx);

                let ty_size = ty.data.byte_size(ctx.arch());

                let offset_value = ir::Value::Expression(ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Multiply),
                    left: index_oper,
                    right: ir::Operand::Constant(ir::Constant::I64(ty_size as i64)),
                });

                let base_oper = Self::val_to_operand(base_address, block, ctx);
                let offset_oper = Self::val_to_operand(offset_value, block, ctx);

                let target_addr_exp = ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: base_oper,
                    right: offset_oper,
                };

                ir::Value::Expression(target_addr_exp)
            }
            Self::StructAccess(StructAccess { base, field, .. }) => {
                let base_ty = base.result_type();
                let (struct_def, _) = base_ty.get_struct_def().unwrap();

                let base_address = base.ir_address(block, ctx);
                let base_address_oper = Self::val_to_operand(base_address, block, ctx);

                let field_offset = struct_def.member_offset(&field.0.data, ctx.arch()).unwrap();

                let target_addr_exp = ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: base_address_oper,
                    right: ir::Operand::Constant(ir::Constant::I64(field_offset as i64)),
                };

                ir::Value::Expression(target_addr_exp)
            }
            other => {
                dbg!(&other);

                todo!("Unknown AExpression for Address Conversion")
            }
        }
    }
}
