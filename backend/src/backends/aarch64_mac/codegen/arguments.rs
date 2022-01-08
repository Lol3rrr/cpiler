use ir::Type;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgTarget {
    GPRegister(u8),
    FPRegister(u8),
    Stack(usize),
}

pub fn arguments<AI, IAI>(arguments: IAI) -> Vec<ArgTarget>
where
    IAI: IntoIterator<IntoIter = AI, Item = Type>,
    AI: Iterator<Item = Type>,
{
    let arguments = arguments.into_iter();
    let mut result = Vec::new();

    let mut ngrn = 0;
    let mut nsrn = 0;
    let mut nsaa = 0;

    for arg in arguments {
        match arg {
            Type::Struct { members } => {
                todo!()
            }
            Type::Float | Type::Double | Type::LongDouble if nsrn < 8 => {
                result.push(ArgTarget::FPRegister(nsrn));
                nsrn += 1;
            }
            Type::Float | Type::Double | Type::LongDouble => {
                todo!()
            }
            other if ngrn < 8 => {
                dbg!(&other);
                result.push(ArgTarget::GPRegister(ngrn));
                ngrn += 1;
            }
            other => {
                let alignment = match &other {
                    Type::Void => panic!(),
                    Type::I8 | Type::U8 => 1,
                    Type::I16 | Type::U16 => 2,
                    Type::I32 | Type::U32 => 4,
                    Type::I64 | Type::U64 => 8,
                    Type::Pointer(_) | Type::Array(_, _) => 8,
                    Type::Struct { .. } | Type::Float | Type::Double | Type::LongDouble => panic!(),
                };
                let size = alignment;

                if nsaa % alignment != 0 {
                    nsaa += alignment - (nsaa % alignment);
                }

                result.push(ArgTarget::Stack(nsaa));
                nsaa += size;
            }
        };
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_register_args() {
        let expected = vec![
            ArgTarget::GPRegister(0),
            ArgTarget::GPRegister(1),
            ArgTarget::GPRegister(2),
        ];

        let result = arguments(vec![Type::I8, Type::I8, Type::I8]);

        assert_eq!(expected, result);
    }

    #[test]
    fn integer_register_spills() {
        let expected = vec![
            ArgTarget::GPRegister(0),
            ArgTarget::GPRegister(1),
            ArgTarget::GPRegister(2),
            ArgTarget::GPRegister(3),
            ArgTarget::GPRegister(4),
            ArgTarget::GPRegister(5),
            ArgTarget::GPRegister(6),
            ArgTarget::GPRegister(7),
            ArgTarget::Stack(0),
            ArgTarget::Stack(2),
        ];

        let result = arguments(vec![
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I8,
            Type::I16,
        ]);

        assert_eq!(expected, result);
    }
}
