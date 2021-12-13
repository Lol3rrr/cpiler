use ir::{BasicBlock, Constant, Statement, Type, Value, Variable};

#[test]
fn single_block_itself() {
    let x_var = Variable::new("x", Type::I8);

    let block = BasicBlock::new(
        vec![],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Constant(Constant::I8(13)),
            },
            Statement::Return(None),
        ],
    );

    let block2 = BasicBlock::new(
        vec![],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Constant(Constant::I8(13)),
            },
            Statement::Return(None),
        ],
    );

    assert_eq!(block, block2);
}

#[test]
fn two_blocks_with_jump() {
    let initial_block1 = BasicBlock::new(vec![], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![Statement::Return(None)],
    );
    initial_block1.add_statement(Statement::Jump(second_block1));

    let initial_block2 = BasicBlock::new(vec![], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![Statement::Return(None)],
    );
    initial_block2.add_statement(Statement::Jump(second_block2));

    assert_eq!(initial_block1, initial_block2);
}

#[test]
fn two_block_cycle_jumps() {
    let initial_block1 = BasicBlock::new(vec![], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![
            Statement::Jump(initial_block1.clone()),
            Statement::Return(None),
        ],
    );
    initial_block1.add_statement(Statement::Jump(second_block1.clone()));
    initial_block1.add_predecessor(second_block1.weak_ptr());

    let initial_block2 = BasicBlock::new(vec![], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![
            Statement::Jump(initial_block2.clone()),
            Statement::Return(None),
        ],
    );
    initial_block2.add_statement(Statement::Jump(second_block2.clone()));
    initial_block2.add_predecessor(second_block2.weak_ptr());

    assert_eq!(initial_block1, initial_block2);
}

#[test]
fn two_block_cycle_jumps_no_equal() {
    let initial_block1 = BasicBlock::new(vec![], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![
            Statement::Jump(initial_block1.clone()),
            Statement::Return(None),
        ],
    );
    initial_block1.add_statement(Statement::Jump(second_block1.clone()));
    initial_block1.add_predecessor(second_block1.weak_ptr());

    let initial_block2 = BasicBlock::new(vec![], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![
            Statement::Jump(initial_block2.clone()),
            Statement::Return(Some(Variable::new("x", Type::I8))),
        ],
    );
    initial_block2.add_statement(Statement::Jump(second_block2.clone()));
    initial_block2.add_predecessor(second_block2.weak_ptr());

    assert_ne!(initial_block1, initial_block2);
}