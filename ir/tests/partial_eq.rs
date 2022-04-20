use ir::{BasicBlock, Constant, JumpMetadata, Statement, Type, Value, Variable};

#[test]
fn single_block_itself() {
    let x_var = Variable::new("x", Type::I8);

    let global = BasicBlock::initial(vec![]);

    let block = BasicBlock::new(
        vec![global.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var.clone(),
                value: Value::Constant(Constant::I8(13)),
            },
            Statement::Return(None),
        ],
    );

    let block2 = BasicBlock::new(
        vec![global.weak_ptr()],
        vec![
            Statement::Assignment {
                target: x_var,
                value: Value::Constant(Constant::I8(13)),
            },
            Statement::Return(None),
        ],
    );

    assert_eq!(block, block2);
}

#[test]
fn two_blocks_with_jump() {
    let global = BasicBlock::initial(vec![]);

    let initial_block1 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![Statement::Return(None)],
    );
    initial_block1.add_statement(Statement::Jump(second_block1, JumpMetadata::Linear));

    let initial_block2 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![Statement::Return(None)],
    );
    initial_block2.add_statement(Statement::Jump(second_block2, JumpMetadata::Linear));

    assert_eq!(initial_block1, initial_block2);
}

#[test]
fn two_block_cycle_jumps() {
    let global = BasicBlock::initial(vec![]);

    let initial_block1 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![
            Statement::Jump(initial_block1.clone(), JumpMetadata::Linear),
            Statement::Return(None),
        ],
    );
    initial_block1.add_statement(Statement::Jump(second_block1.clone(), JumpMetadata::Linear));
    initial_block1.add_predecessor(second_block1.weak_ptr());

    let initial_block2 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![
            Statement::Jump(initial_block2.clone(), JumpMetadata::Linear),
            Statement::Return(None),
        ],
    );
    initial_block2.add_statement(Statement::Jump(second_block2.clone(), JumpMetadata::Linear));
    initial_block2.add_predecessor(second_block2.weak_ptr());

    assert_eq!(initial_block1, initial_block2);
}

#[test]
fn two_block_cycle_jumps_no_equal() {
    let global = BasicBlock::initial(vec![]);

    let initial_block1 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block1 = BasicBlock::new(
        vec![initial_block1.weak_ptr()],
        vec![
            Statement::Jump(initial_block1.clone(), JumpMetadata::Linear),
            Statement::Return(None),
        ],
    );
    initial_block1.add_statement(Statement::Jump(second_block1.clone(), JumpMetadata::Linear));
    initial_block1.add_predecessor(second_block1.weak_ptr());

    let initial_block2 = BasicBlock::new(vec![global.weak_ptr()], vec![]);
    let second_block2 = BasicBlock::new(
        vec![initial_block2.weak_ptr()],
        vec![
            Statement::Jump(initial_block2.clone(), JumpMetadata::Linear),
            Statement::Return(Some(Variable::new("x", Type::I8))),
        ],
    );
    initial_block2.add_statement(Statement::Jump(second_block2.clone(), JumpMetadata::Linear));
    initial_block2.add_predecessor(second_block2.weak_ptr());

    assert_ne!(initial_block1, initial_block2);
}
