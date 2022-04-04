use general::{Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Statement, Type, Value, Variable,
};
use optimizer::{Optimization, OptimizationPass};

#[test]
fn unused_if_statement() {
    let content = "
void test() {
    long x = 0;
    if (0) {
        int tmp = 0;
    }
    x = 1;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let semantic_ast = semantic::parse(syntax_ast).unwrap();
    let raw_ir = semantic_ast.convert_to_ir(general::arch::Arch::X86_64);

    let tmp_var = Variable::new("tmp", Type::I32);
    let x0_var = Variable::new("x_17563920617334630623", Type::I64);
    let x1_var = x0_var.next_gen();

    let expected_global = BasicBlock::initial(vec![]);
    let expected_func_start = BasicBlock::new(vec![expected_global.weak_ptr()], vec![]);

    let expected_func_first = BasicBlock::new(
        vec![expected_func_start.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x0_var.clone(),
                value: Value::Constant(Constant::I64(0)),
            },
            Statement::SaveVariable { var: x0_var },
        ],
    );
    expected_func_start.add_statement(Statement::Jump(
        expected_func_first.clone(),
        ir::JumpMetadata::Linear,
    ));

    let expected_missing_block = BasicBlock::new(
        vec![expected_func_first.weak_ptr()],
        vec![Statement::Assignment {
            target: tmp_var,
            value: Value::Expression(Expression::Cast {
                target: Type::I32,
                base: Operand::Constant(Constant::I64(0)),
            }),
        }],
    );

    let expected_func_second = BasicBlock::new(
        vec![
            expected_func_first.weak_ptr(),
            expected_missing_block.weak_ptr(),
        ],
        vec![
            Statement::Assignment {
                target: x1_var.clone(),
                value: Value::Constant(Constant::I64(1)),
            },
            Statement::SaveVariable { var: x1_var },
        ],
    );
    expected_func_first.add_statement(Statement::Jump(
        expected_func_second,
        ir::JumpMetadata::Linear,
    ));

    let expected = FunctionDefinition {
        name: "test".to_string(),
        arguments: vec![],
        return_ty: Type::Void,
        block: expected_func_start,
    };

    let func = raw_ir.functions.get("test").unwrap().clone();
    dbg!(&func);
    let dead_code_pass = optimizer::optimizations::DeadCode::new().repeat(2);
    let result = dead_code_pass.pass_function(func);
    dbg!(&result);

    assert_eq!(expected, result);
}
