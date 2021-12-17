use std::path::{Path, PathBuf};

use general::{Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Program, Statement, Type, Value,
    Variable,
};

#[test]
fn simple_while_loop() {
    let input = "
void test() {
    while (2) {
        int x = 0;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(func_first_block.clone()));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(loop_cond_block.clone()));

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::Jump(loop_cond_block.clone()),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var.clone(),
        loop_inner_block.clone(),
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());

    let func_end_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![Statement::Return(None)],
    );
    loop_cond_block.add_statement(Statement::Jump(func_end_block.clone()));

    let expected = Program {
        global: global.clone(),
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("simple-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
#[ignore = "This is currently not yet supported"]
fn while_loop_modifying_in_cond_inner() {
    let input = "
void test() {
    int x = 0;
    while (x--) {
        x = x - 1;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(func_first_block.clone()));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(loop_cond_block.clone()));

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::Jump(loop_cond_block.clone()),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var.clone(),
        loop_inner_block.clone(),
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());

    let func_end_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![Statement::Return(None)],
    );
    loop_cond_block.add_statement(Statement::Jump(func_end_block.clone()));

    let expected = Program {
        global: global.clone(),
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("modifying-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn while_loop_with_break() {
    let input = "
void test() {
    while (2) {
        int x = 0;
        break;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(func_first_block.clone()));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(loop_cond_block.clone()));

    let func_end_block = BasicBlock::new(vec![], vec![Statement::Return(None)]);

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::Jump(func_end_block.clone()),
            Statement::Jump(loop_cond_block.clone()),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var.clone(),
        loop_inner_block.clone(),
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());
    loop_cond_block.add_statement(Statement::Jump(func_end_block.clone()));

    func_end_block.add_predecessor(loop_cond_block.weak_ptr());
    func_end_block.add_predecessor(loop_inner_block.weak_ptr());

    let expected = Program {
        global: global.clone(),
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    dbg!(&expected);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("break-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}

#[test]
fn while_loop_with_continue() {
    let input = "
void test() {
    while (2) {
        int x = 0;
        continue;
    }

    return;
}
        ";
    let source = Source::new("test", input);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let input = semantic::parse(syntax_ast).unwrap();

    let global = BasicBlock::initial(vec![]);

    let t0_var = Variable::new("__t_0", Type::I64);
    let x0_var = Variable::new("x", Type::I32);

    let func_initial_block = BasicBlock::new(vec![global.weak_ptr()], vec![]);

    let func_first_block = BasicBlock::new(vec![func_initial_block.weak_ptr()], vec![]);
    func_initial_block.add_statement(Statement::Jump(func_first_block.clone()));

    let loop_cond_block = BasicBlock::new(
        vec![func_first_block.weak_ptr()],
        vec![Statement::Assignment {
            target: t0_var.clone(),
            value: Value::Constant(Constant::I64(2)),
        }],
    );
    func_first_block.add_statement(Statement::Jump(loop_cond_block.clone()));

    let func_end_block = BasicBlock::new(vec![], vec![Statement::Return(None)]);

    let loop_inner_block = BasicBlock::new(
        vec![loop_cond_block.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Expression(Expression::Cast {
                    target: Type::I32,
                    base: Operand::Constant(Constant::I64(0)),
                }),
            },
            Statement::Jump(loop_cond_block.clone()),
            Statement::Jump(loop_cond_block.clone()),
        ],
    );
    loop_cond_block.add_statement(Statement::JumpTrue(
        t0_var.clone(),
        loop_inner_block.clone(),
    ));
    loop_cond_block.add_predecessor(loop_inner_block.weak_ptr());
    loop_cond_block.add_statement(Statement::Jump(func_end_block.clone()));

    func_end_block.add_predecessor(loop_cond_block.weak_ptr());

    let expected = Program {
        global: global.clone(),
        functions: vec![(
            "test".to_string(),
            FunctionDefinition {
                name: "test".to_string(),
                arguments: vec![],
                return_ty: Type::Void,
                block: func_initial_block,
            },
        )]
        .into_iter()
        .collect(),
    };

    let result = input.convert_to_ir();
    dbg!(&result);

    dbg!(&expected);

    let test_file_path = {
        let raw = Path::new(file!());
        let mut result = PathBuf::new();
        for part in raw.components().skip(1) {
            result.push(part);
        }

        result
    };
    let test_dir_path = test_file_path.parent().unwrap();

    let result_path = test_dir_path.join("continue-result.dot");
    std::fs::write(result_path, result.to_dot()).unwrap();

    assert_eq!(expected, result);
}
