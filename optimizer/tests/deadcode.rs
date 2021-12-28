use general::{Source, Span};
use ir::{
    BasicBlock, Constant, Expression, FunctionDefinition, Operand, Statement, Type, Value, Variable,
};
use optimizer::{Optimization, OptimizationPass};

#[test]
fn unused_if_statement() {
    let content = "
void test() {
    int x = 0;
    if (0) {
        int tmp = 0;
    }
    x = 1;
}
        ";
    let source = Source::new("test", content);
    let span: Span = source.clone().into();
    let tokens = tokenizer::tokenize(span);
    let syntax_ast = syntax::parse(tokens).unwrap();
    let semantic_ast = semantic::parse(syntax_ast).unwrap();
    let raw_ir = semantic_ast.convert_to_ir(general::arch::Arch::X86_64);

    let tmp_var = Variable::new("tmp", Type::I32);

    let expected_global = BasicBlock::initial(vec![]);
    let expected_func_start = BasicBlock::new(vec![expected_global.weak_ptr()], vec![]);

    let expected_func_first = BasicBlock::new(vec![expected_func_start.weak_ptr()], vec![]);
    expected_func_start.add_statement(Statement::Jump(expected_func_first.clone()));

    let expected_missing_block = BasicBlock::new(
        vec![expected_func_first.weak_ptr()],
        vec![Statement::Assignment {
            target: tmp_var.clone(),
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
        vec![],
    );
    expected_func_first.add_statement(Statement::Jump(expected_func_second));

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
