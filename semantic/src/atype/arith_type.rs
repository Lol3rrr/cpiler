use crate::{AExpression, APrimitive, AType, SemanticError};

pub fn determine_types(
    left: AExpression,
    right: AExpression,
) -> Result<(AExpression, AExpression), SemanticError> {
    let left_type = left.result_type();
    let right_type = right.result_type();
    if left_type == right_type {
        return Ok((left, right));
    }

    let left_prim = match left_type {
        AType::Primitve(prim) => prim,
        _ => {
            return Err(SemanticError::MismatchedOperationTypes {
                left: general::SpanData {
                    span: left.entire_span(),
                    data: left_type,
                },
                right: general::SpanData {
                    span: right.entire_span(),
                    data: right_type,
                },
            })
        }
    };
    let right_prim = match right_type {
        AType::Primitve(prim) => prim,
        _ => panic!("Different Types with more Complex DataTypes"),
    };

    match (left_prim, right_prim) {
        (APrimitive::LongDouble, _) => {
            let n_exp = AExpression::Cast {
                base: Box::new(right),
                target: AType::Primitve(APrimitive::LongDouble),
            };

            Ok((left, n_exp))
        }
        (_, APrimitive::LongDouble) => {
            let n_exp = AExpression::Cast {
                base: Box::new(left),
                target: AType::Primitve(APrimitive::LongDouble),
            };

            Ok((n_exp, right))
        }
        (APrimitive::Double, _) => {
            let n_exp = AExpression::Cast {
                base: Box::new(right),
                target: AType::Primitve(APrimitive::Double),
            };

            Ok((left, n_exp))
        }
        (_, APrimitive::Double) => {
            let n_exp = AExpression::Cast {
                base: Box::new(left),
                target: AType::Primitve(APrimitive::Double),
            };

            Ok((n_exp, right))
        }
        (APrimitive::Float, _) => {
            let n_exp = AExpression::Cast {
                base: Box::new(right),
                target: AType::Primitve(APrimitive::Float),
            };

            Ok((left, n_exp))
        }
        (_, APrimitive::Float) => {
            let n_exp = AExpression::Cast {
                base: Box::new(left),
                target: AType::Primitve(APrimitive::Float),
            };

            Ok((n_exp, right))
        }
        (left_prim, right_prim) if left_prim.is_unsigned() && right_prim.is_unsigned() => {
            if left_prim.rank() > right_prim.rank() {
                let n_exp = AExpression::Cast {
                    target: AType::Primitve(left_prim),
                    base: Box::new(right),
                };

                Ok((left, n_exp))
            } else {
                let n_exp = AExpression::Cast {
                    target: AType::Primitve(right_prim),
                    base: Box::new(left),
                };

                Ok((n_exp, right))
            }
        }
        (left_prim, right_prim) if left_prim.is_signed() && right_prim.is_signed() => {
            if left_prim.rank() > right_prim.rank() {
                let n_exp = AExpression::Cast {
                    target: AType::Primitve(left_prim),
                    base: Box::new(right),
                };

                Ok((left, n_exp))
            } else {
                let n_exp = AExpression::Cast {
                    target: AType::Primitve(right_prim),
                    base: Box::new(left),
                };

                Ok((n_exp, right))
            }
        }
        (left_prim, right_prim) => {
            let left_rank = left_prim.rank().unwrap();
            let right_rank = right_prim.rank().unwrap();

            if left_rank >= right_rank && left_prim.is_unsigned() || left_rank > right_rank {
                let n_right = AExpression::Cast {
                    base: Box::new(right),
                    target: AType::Primitve(left_prim),
                };

                return Ok((left, n_right));
            } else if right_rank >= left_rank && right_prim.is_unsigned() || right_rank > left_rank
            {
                let n_left = AExpression::Cast {
                    base: Box::new(left),
                    target: AType::Primitve(right_prim),
                };

                return Ok((n_left, right));
            }
            dbg!(left_prim.rank(), right_prim.rank());

            todo!("Different Primitives");
        }
    }
}

#[cfg(test)]
mod tests {
    use general::{Source, Span, SpanData};

    use crate::{APrimitive, Literal};

    use super::*;

    #[test]
    fn same_types() {
        let in_source = Source::new("test", "123 + 234");
        let left_in = AExpression::Literal(Literal::Integer(SpanData {
            span: Span::new_source(in_source.clone(), 0..3),
            data: 123,
        }));
        let right_in = AExpression::Literal(Literal::Integer(SpanData {
            span: Span::new_source(in_source.clone(), 6..9),
            data: 234,
        }));

        let expected_left = left_in.clone();
        let expected_right = right_in.clone();
        let expected = Ok((expected_left, expected_right));

        let result = determine_types(left_in, right_in);
        dbg!(&result);

        assert_eq!(expected, result);
    }

    #[test]
    fn float_int() {
        let in_source = Source::new("test", "1.3 + 234");
        let left_in = AExpression::Literal(Literal::FloatingPoint(SpanData {
            span: Span::new_source(in_source.clone(), 0..3),
            data: 1.3,
        }));
        let right_in = AExpression::Literal(Literal::Integer(SpanData {
            span: Span::new_source(in_source.clone(), 6..9),
            data: 234,
        }));

        let expected_left = left_in.clone();
        let expected_right = AExpression::Cast {
            base: Box::new(right_in.clone()),
            target: AType::Primitve(APrimitive::Float),
        };
        let expected = Ok((expected_left, expected_right));

        let result = determine_types(left_in, right_in);
        dbg!(&result);

        assert_eq!(expected, result);
    }

    #[test]
    fn uint_int() {
        let source = Source::new("test", "unsigned int");
        let left_in = AExpression::Literal(Literal::Integer(SpanData {
            span: Span::new_source(source.clone(), 9..12),
            data: 1,
        }));
        let right_in = AExpression::Cast {
            base: Box::new(AExpression::Literal(Literal::Integer(SpanData {
                span: Span::new_source(source.clone(), 0..12),
                data: 2,
            }))),
            target: AType::Primitve(APrimitive::UnsignedLongInt),
        };

        let expected_left = AExpression::Cast {
            base: Box::new(left_in.clone()),
            target: AType::Primitve(APrimitive::UnsignedLongInt),
        };
        let expected_right = right_in.clone();
        let expected = Ok((expected_left, expected_right));

        let result = determine_types(left_in, right_in);
        dbg!(&result);

        assert_eq!(expected, result);
    }
    #[test]
    fn int_uint() {
        let source = Source::new("test", "unsigned int");
        let right_in = AExpression::Literal(Literal::Integer(SpanData {
            span: Span::new_source(source.clone(), 9..12),
            data: 1,
        }));
        let left_in = AExpression::Cast {
            base: Box::new(AExpression::Literal(Literal::Integer(SpanData {
                span: Span::new_source(source.clone(), 0..12),
                data: 2,
            }))),
            target: AType::Primitve(APrimitive::UnsignedLongInt),
        };

        let expected_right = AExpression::Cast {
            base: Box::new(right_in.clone()),
            target: AType::Primitve(APrimitive::UnsignedLongInt),
        };
        let expected_left = left_in.clone();
        let expected = Ok((expected_left, expected_right));

        let result = determine_types(left_in, right_in);
        dbg!(&result);

        assert_eq!(expected, result);
    }
}
