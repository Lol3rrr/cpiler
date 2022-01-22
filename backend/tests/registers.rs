use backend::util::registers;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
enum TestRegister {
    GeneralPurpose(u8),
    FloatingPoint(u8),
}

impl registers::Register for TestRegister {
    fn reg_type(&self) -> registers::RegisterType {
        match self {
            Self::GeneralPurpose(_) => registers::RegisterType::GeneralPurpose,
            Self::FloatingPoint(_) => registers::RegisterType::FloatingPoint,
        }
    }

    fn align_size(&self) -> (usize, usize) {
        match self {
            Self::GeneralPurpose(_) => (4, 4),
            Self::FloatingPoint(_) => (4, 4),
        }
    }
}

mod linear {
    use super::*;

    #[test]
    fn no_spill() {
        let test_program = "
int main() {
    int x = 0;
    int y = x + 3;
    return y;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = registers::allocate_registers(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
            ],
        );
        dbg!(&result_allocation);

        // TODO
    }

    #[test]
    fn spill() {
        let test_program = "
int main() {
    int x = 0;
    int y = 3;
    int z = x + y;
    int w = x + y;
    return x + z;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = registers::allocate_registers(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }
}

mod conditional {
    use super::*;

    #[test]
    fn spill() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;

    if (1) {
        z = 2 + 6;
        w = 3 + 5;
    } else {
        w = 2 + 5;
    }

    return z + w;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = registers::allocate_registers(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
                TestRegister::GeneralPurpose(4),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }
}

mod loops {
    use super::*;

    #[test]
    fn spill() {
        let test_program = "
int main() {
    int z = 0;
    int w = 0;

    while (1) {
        z = 2 + 6;
        w = 3 + 7;
    }

    return z + w;
}
        ";
        let test_source = general::Source::new("test", test_program);
        let test_span: general::Span = test_source.clone().into();
        let tokens = tokenizer::tokenize(test_span);
        let ast = syntax::parse(tokens).unwrap();
        let aast = semantic::parse(ast).unwrap();
        let ir = aast.convert_to_ir(general::arch::Arch::X86_64);

        dbg!(&ir);

        let main_func: ir::FunctionDefinition = ir.functions.get("main").unwrap().clone();

        let result_allocation = registers::allocate_registers(
            &main_func,
            &[
                TestRegister::GeneralPurpose(0),
                TestRegister::GeneralPurpose(1),
                TestRegister::GeneralPurpose(2),
                TestRegister::GeneralPurpose(3),
            ],
        );
        dbg!(&result_allocation);

        dbg!(&main_func);

        // TODO
    }
}
