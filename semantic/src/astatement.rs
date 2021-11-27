use general::{Span, SpanData};
use syntax::{AssignTarget, Identifier, Statement};

use crate::{
    AExpression, APrimitive, AScope, AType, ParseState, SemanticError, TypeDefinitions,
    VariableContainer,
};

#[derive(Debug, PartialEq)]
pub enum AAssignTarget {
    Variable {
        ident: Identifier,
        /// The Type of the Variable itself
        ty_info: SpanData<AType>,
    },
    ArrayAccess {
        target: Box<Self>,
        index: AExpression,
        /// The Type of the Array-Element
        ty_info: SpanData<AType>,
    },
    StructField {
        target: Box<Self>,
        field: Identifier,
        /// The Type of the Field of the Struct
        ty_info: SpanData<AType>,
    },
}

#[derive(Debug, PartialEq)]
pub enum AStatement {
    Assignment {
        target: AAssignTarget,
        value: AExpression,
    },
    Expression(AExpression),
    WhileLoop {
        condition: AExpression,
        body: AScope,
    },
    Break,
    If {
        condition: AExpression,
        body: AScope,
    },
    Return {
        value: Option<AExpression>,
    },
}

impl AAssignTarget {
    pub fn parse<VC>(raw: AssignTarget, vars: &VC) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            AssignTarget::Variable(ident) => {
                let (ty, sp) = match vars.get_var(&ident) {
                    Some(t) => t,
                    None => return Err(SemanticError::UnknownIdentifier { name: ident }),
                };

                Ok(AAssignTarget::Variable {
                    ident,
                    ty_info: SpanData {
                        span: sp.clone(),
                        data: ty.clone(),
                    },
                })
            }
            AssignTarget::ArrayAccess { base, index } => {
                let base_target = Self::parse(*base, vars)?;

                let a_index = AExpression::parse(index, vars)?;

                let index_type = a_index.result_type();
                match index_type {
                    AType::Primitve(APrimitive::Int) => {}
                    i_type => {
                        let i_span = a_index.entire_span();
                        dbg!(&i_type, &i_span);

                        todo!("Invalid Index-Type");
                    }
                };

                let (arr_ty, arr_span) = base_target.get_expected_type();
                let elem_ty = match arr_ty {
                    AType::Array(base) => base.ty,
                    other => {
                        dbg!(&other);

                        todo!("Indexed type is not an array");
                    }
                };

                // TODO
                // This span is actually pretty bad because as far as I can tell it just describes
                // the name of the array Variable and not the underlying Type
                let elem_span = arr_span;

                Ok(AAssignTarget::ArrayAccess {
                    target: Box::new(base_target),
                    index: a_index,
                    ty_info: SpanData {
                        span: elem_span,
                        data: *elem_ty,
                    },
                })
            }
            AssignTarget::StructAccess { base, field } => {
                dbg!(&base, &field);

                let base_target = Self::parse(*base, vars)?;
                dbg!(&base_target);

                let (base_ty, _) = base_target.get_expected_type();
                let struct_def = match base_ty {
                    AType::AnonStruct(def) => def,
                    other => {
                        dbg!(&other);

                        todo!("Wrong Type, expected a struct");
                    }
                };
                dbg!(&struct_def);

                let (field_ty, field_span) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        todo!("Unknown field")
                    }
                };
                dbg!(&field_ty);

                Ok(Self::StructField {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty.clone(),
                    },
                })
            }
            unknown => panic!("Unexpected Assignment Target: {:?}", unknown),
        }
    }

    pub fn get_expected_type(&self) -> (AType, Span) {
        match &self {
            Self::Variable { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::ArrayAccess { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::StructField { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
        }
    }
}

impl AStatement {
    pub fn parse<VC>(
        raw: Statement,
        parse_state: &ParseState,
        vars: &VC,
        types: &TypeDefinitions,
    ) -> Result<Self, SemanticError>
    where
        VC: VariableContainer,
    {
        match raw {
            Statement::VariableAssignment { target, value } => {
                let value_exp = AExpression::parse(value, vars)?;

                let a_target = AAssignTarget::parse(target, vars)?;

                let (var_type, var_span) = a_target.get_expected_type();
                let exp_type = value_exp.result_type();
                if var_type != exp_type {
                    return Err(SemanticError::MismatchedTypes {
                        expected: SpanData {
                            span: var_span,
                            data: var_type,
                        },
                        received: SpanData {
                            span: value_exp.entire_span(),
                            data: exp_type,
                        },
                    });
                }

                Ok(Self::Assignment {
                    target: a_target,
                    value: value_exp,
                })
            }
            Statement::SingleExpression(raw_exp) => {
                let exp = AExpression::parse(raw_exp, vars)?;

                Ok(Self::Expression(exp))
            }
            Statement::WhileLoop { condition, scope } => {
                dbg!(&condition, &scope);

                let cond = AExpression::parse(condition, vars)?;
                dbg!(&cond);

                let inner_scope = AScope::parse(parse_state, scope)?;
                dbg!(&inner_scope);

                Ok(Self::WhileLoop {
                    condition: cond,
                    body: inner_scope,
                })
            }
            Statement::Break => Ok(Self::Break),
            Statement::If {
                condition,
                scope,
                elses,
            } => {
                dbg!(&condition, &scope, &elses);

                let cond = AExpression::parse(condition, vars)?;
                dbg!(&cond);

                let inner_scope = AScope::parse(parse_state, scope)?;
                dbg!(&inner_scope);

                // TODO
                // Parse Elses

                Ok(Self::If {
                    condition: cond,
                    body: inner_scope,
                })
            }
            Statement::Return(raw_val) => {
                dbg!(&raw_val);
                let r_value = match raw_val {
                    Some(raw) => {
                        let value = AExpression::parse(raw, vars)?;

                        Some(value)
                    }
                    None => None,
                };

                Ok(Self::Return { value: r_value })
            }
            unknown => panic!("Unexpected Statement: {:?}", unknown),
        }
    }
}
