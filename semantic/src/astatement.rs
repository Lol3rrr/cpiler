use general::{Span, SpanData};
use syntax::{AssignTarget, Identifier, Statement};

use crate::{
    AExpression, AScope, AType, ParseState, SemanticError, TypeDefinitions, VariableContainer,
};

#[derive(Debug, PartialEq)]
pub enum AAssignTarget {
    Variable(Identifier),
    StructField {
        target: Box<Self>,
        field: Identifier,
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
                let (var_type, var_dec) =
                    vars.get_var(&ident)
                        .ok_or_else(|| SemanticError::UnknownIdentifier {
                            name: ident.clone(),
                        })?;

                Ok(AAssignTarget::Variable(ident))
            }
            AssignTarget::StructAccess { base, field } => {
                dbg!(&base, &field);

                todo!("Handle struct field assignment");
            }
            unknown => panic!("Unexpected Assignment Target: {:?}", unknown),
        }
    }

    pub fn get_expected_type<VC>(&self, vars: &VC) -> (AType, Span)
    where
        VC: VariableContainer,
    {
        match &self {
            Self::Variable(ident) => vars
                .get_var(ident)
                .map(|(t, s)| (t.clone(), s.clone()))
                .unwrap(),
            Self::StructField { target, field } => {
                dbg!(&target, &field);

                todo!("Get Type of Struct-Field");
            }
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

                let (var_type, var_span) = a_target.get_expected_type(vars);
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
