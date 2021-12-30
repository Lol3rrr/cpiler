use general::{Span, SpanData};
use ir::BasicBlock;
use syntax::{AssignTarget, Identifier};

use crate::{
    conversion::ConvertContext, AExpression, APrimitive, AType, SemanticError, TypeDefinitions,
    VariableContainer,
};

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayAccessTarget {
    pub target: Box<AAssignTarget>,
    pub index: Box<AExpression>,
    pub ty_info: SpanData<AType>,
}

impl ArrayAccessTarget {
    pub fn to_exp(self, block: &mut BasicBlock, ctx: &ConvertContext) -> ir::Expression {
        let (base_address, _) = self.target.base_target_address(block, ctx);

        let elem_size = self.ty_info.data.byte_size(ctx.arch());

        let index_value = self.index.to_ir(block, ctx);
        let index_oper = AExpression::val_to_operand(index_value, block, ctx);
        dbg!(&index_oper);

        let offset_exp = ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Multiply),
            left: index_oper,
            right: ir::Operand::Constant(ir::Constant::I64(elem_size as i64)),
        };
        let offset_value = ir::Value::Expression(offset_exp);
        let offset_oper = AExpression::val_to_operand(offset_value, block, ctx);

        let base_oper = AExpression::val_to_operand(base_address, block, ctx);

        ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
            left: base_oper,
            right: offset_oper,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructFieldTarget {
    /// The underlying Struct Target, like a Variable or Pointer
    pub target: Box<AAssignTarget>,
    /// The Name of the Field
    pub field: Identifier,
    /// The Type of the Field of the Struct
    pub ty_info: SpanData<AType>,
}

impl StructFieldTarget {
    pub fn to_exp(self, block: &mut BasicBlock, ctx: &ConvertContext) -> ir::Expression {
        let (base_value, base_ty) = self.target.base_target_address(block, ctx);
        dbg!(&base_value, &base_ty);

        let struct_def = match base_ty {
            AType::Struct { def, .. } => def,
            other => {
                dbg!(&other);

                panic!("Expected Struct as Target Type")
            }
        };

        let offset = struct_def
            .member_offset(&self.field.0.data, ctx.arch())
            .expect("Field does not exist");

        let base_oper = AExpression::val_to_operand(base_value, block, ctx);
        dbg!(&base_oper);

        ir::Expression::BinaryOp {
            op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
            left: base_oper,
            right: ir::Operand::Constant(ir::Constant::I64(offset as i64)),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AAssignTarget {
    Variable {
        ident: Identifier,
        /// The Type of the Variable itself
        ty_info: SpanData<AType>,
    },
    Deref {
        /// The Target Address to write the Value to
        exp: AExpression,
        /// The Type that is stored that that Address
        ty_info: SpanData<AType>,
    },
    ArrayAccess(ArrayAccessTarget),
    StructField(StructFieldTarget),
}

impl AAssignTarget {
    fn ty_span(&self) -> &Span {
        match self {
            Self::Variable { ty_info, .. } => &ty_info.span,
            Self::Deref { ty_info, .. } => &ty_info.span,
            Self::ArrayAccess(arr) => &arr.ty_info.span,
            Self::StructField(str_tar) => &str_tar.ty_info.span,
        }
    }

    fn base_ty(&self) -> &AType {
        match self {
            Self::Variable { ty_info, .. } => &ty_info.data,
            Self::Deref { ty_info, .. } => &ty_info.data,
            Self::ArrayAccess(ArrayAccessTarget { ty_info, .. }) => &ty_info.data,
            Self::StructField(StructFieldTarget { ty_info, .. }) => &ty_info.data,
        }
    }

    pub fn parse<VC>(
        raw: AssignTarget,
        ty_defs: &TypeDefinitions,
        vars: &VC,
    ) -> Result<Self, SemanticError>
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
                let base_target = Self::parse(*base, ty_defs, vars)?;

                let a_index = AExpression::parse(index, ty_defs, vars)?;

                let index_type = a_index.result_type();
                match index_type {
                    AType::Primitve(APrimitive::Int) | AType::Primitve(APrimitive::LongInt) => {}
                    i_type => {
                        let i_span = a_index.entire_span();
                        dbg!(&i_type, &i_span);

                        todo!("Invalid Index-Type");
                    }
                };

                let (arr_ty, arr_span) = base_target.get_expected_type();
                let elem_ty = match arr_ty {
                    AType::Array(base) => base.ty,
                    AType::Pointer(base) => base,
                    other => {
                        dbg!(&other);

                        todo!("Indexed type is not an array");
                    }
                };

                // TODO
                // This span is actually pretty bad because as far as I can tell it just describes
                // the name of the array Variable and not the underlying Type
                let elem_span = arr_span;

                Ok(AAssignTarget::ArrayAccess(ArrayAccessTarget {
                    target: Box::new(base_target),
                    index: Box::new(a_index),
                    ty_info: SpanData {
                        span: elem_span,
                        data: *elem_ty,
                    },
                }))
            }
            AssignTarget::StructAccess { base, field } => {
                let base_target = Self::parse(*base, ty_defs, vars)?;

                let (base_ty, _) = base_target.get_expected_type();

                let (struct_def, def_span) = match base_ty.get_struct_def() {
                    Some(s) => s,
                    None => {
                        let span = base_target.ty_span().clone();

                        return Err(SemanticError::StructAccessOnNonStruct {
                            field_name: field,
                            received: SpanData {
                                span,
                                data: base_ty,
                            },
                        });
                    }
                };

                let (field_ty, field_span) = match struct_def.find_member(&field) {
                    Some(f) => (f.data, f.span),
                    None => {
                        return Err(SemanticError::UnknownStructField {
                            field_name: field.clone(),
                            struct_def: SpanData {
                                span: def_span.clone(),
                                data: struct_def.clone(),
                            },
                        });
                    }
                };

                Ok(Self::StructField(StructFieldTarget {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                }))
            }
            AssignTarget::StructPtrAccess { base, field } => {
                dbg!(&base, &field);

                let base_target = Self::parse(*base, ty_defs, vars)?;
                dbg!(&base_target);

                let (struct_def, def_span) = match base_target.base_ty() {
                    AType::Pointer(inner) => match inner.get_struct_def() {
                        Some(s) => s,
                        None => {
                            dbg!(&inner);

                            todo!("Expected a Struct-Pointer");
                        }
                    },
                    other => {
                        dbg!(&other);

                        todo!("Expected a Struct-Pointer");
                    }
                };
                dbg!(&struct_def);

                let (field_ty, field_span) = match struct_def.find_member(&field) {
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
                dbg!(&field_ty);

                Ok(Self::StructField(StructFieldTarget {
                    target: Box::new(base_target),
                    field,
                    ty_info: SpanData {
                        span: field_span,
                        data: field_ty,
                    },
                }))
            }
        }
    }

    pub fn get_expected_type(&self) -> (AType, Span) {
        match &self {
            Self::Variable { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::Deref { ty_info, .. } => (ty_info.data.clone(), ty_info.span.clone()),
            Self::ArrayAccess(ArrayAccessTarget { ty_info, .. }) => {
                (ty_info.data.clone(), ty_info.span.clone())
            }
            Self::StructField(StructFieldTarget { ty_info, .. }) => {
                (ty_info.data.clone(), ty_info.span.clone())
            }
        }
    }

    /// Returns an ir::Value that contains the base Address of the Target, this means when you need
    /// to calculate the Address for an Array access you can use this function to load the Base
    /// starting Address of the Array and then add the needed offset for the Index
    pub fn base_target_address(
        self,
        block: &mut BasicBlock,
        ctx: &ConvertContext,
    ) -> (ir::Value, AType) {
        match self {
            Self::Variable { ident, ty_info } => {
                dbg!(&ty_info);

                match ty_info.data.ty() {
                    AType::Struct { def, area } => {
                        let var = AExpression::Variable {
                            ident,
                            ty: SpanData {
                                span: ty_info.span,
                                data: AType::Struct {
                                    def: def.clone(),
                                    area: area.clone(),
                                },
                            },
                        };

                        (var.to_ir(block, ctx), AType::Struct { def, area })
                    }
                    AType::Array(arr) => {
                        let var = AExpression::Variable {
                            ident,
                            ty: SpanData {
                                span: ty_info.span,
                                data: AType::Array(arr.clone()),
                            },
                        };

                        (var.to_ir(block, ctx), AType::Array(arr))
                    }
                    AType::Pointer(base_ty) => {
                        let var = AExpression::Variable {
                            ident,
                            ty: SpanData {
                                span: ty_info.span,
                                data: AType::Pointer(base_ty.clone()),
                            },
                        };

                        (var.to_ir(block, ctx), AType::Pointer(base_ty))
                    }
                    other => {
                        dbg!(&other);

                        todo!("Variable of different Type");
                    }
                }
            }
            Self::ArrayAccess(arr_target) => {
                dbg!(&arr_target);

                let target_ty = arr_target.ty_info.data.into_ty().clone();

                let target_exp = arr_target.to_exp(block, ctx);

                (ir::Value::Expression(target_exp), target_ty)
            }
            Self::StructField(StructFieldTarget {
                target,
                field,
                ty_info,
            }) => {
                dbg!(&target, &field, &ty_info);

                let (base_address_value, base_target_ty) = target.base_target_address(block, ctx);
                dbg!(&base_address_value, &base_target_ty);
                let base_address_oper = AExpression::val_to_operand(base_address_value, block, ctx);

                let (struct_def, _) = base_target_ty.get_struct_def().unwrap();

                let raw_offset = struct_def.member_offset(&field.0.data, ctx.arch()).unwrap();
                let offset_oper = ir::Operand::Constant(ir::Constant::I64(raw_offset as i64));

                let target_exp = ir::Expression::BinaryOp {
                    op: ir::BinaryOp::Arith(ir::BinaryArithmeticOp::Add),
                    left: base_address_oper,
                    right: offset_oper,
                };
                (ir::Value::Expression(target_exp), ty_info.data)
            }
            other => {
                dbg!(&other);

                todo!("Non Variable Target");
            }
        }
    }
}
