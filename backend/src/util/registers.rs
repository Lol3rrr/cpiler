//! Handles all the Register allocation and the like
// This is build around this Paper
// https://link.springer.com/content/pdf/10.1007%2F11688839_20.pdf

use std::{collections::HashMap, fmt::Debug, hash::Hash, path::PathBuf};

use ir::Variable;

/// This will perform the Register Allocation and spilling
pub fn allocate_registers<R>(
    func: &ir::FunctionDefinition,
    registers: &[R],
    build_path: Option<PathBuf>,
) -> HashMap<Variable, R>
where
    R: Clone + Debug + Hash + PartialEq + Eq + register_allocation::Register,
{
    register_allocation::RegisterMapping::allocate(
        func,
        registers,
        register_allocation::AllocationCtx { build_path },
    )
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
    enum TestRegister {
        General(u8),
        Float(u8),
    }

    impl register_allocation::Register for TestRegister {
        fn reg_type(&self) -> register_allocation::RegisterType {
            match self {
                Self::General(_) => register_allocation::RegisterType::GeneralPurpose,
                Self::Float(_) => register_allocation::RegisterType::FloatingPoint,
            }
        }

        fn align_size(&self) -> (usize, usize) {
            (4, 4)
        }
    }

    #[test]
    #[ignore = "For some reason"]
    fn fits() {
        let input_register = vec![TestRegister::General(0)];
        let input_statements = vec![ir::Statement::Assignment {
            target: ir::Variable::new("test", ir::Type::U8),
            value: ir::Value::Unknown,
        }];

        let input_function = ir::FunctionDefinition {
            name: "test".to_string(),
            block: ir::BasicBlock::new(vec![], input_statements),
            arguments: vec![],
            return_ty: ir::Type::Void,
        };

        let _ = allocate_registers(&input_function, &input_register, None);
    }
}
