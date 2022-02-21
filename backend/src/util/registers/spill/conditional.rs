use crate::util::registers::spill::replace;

fn insert_load(start_block: ir::BasicBlock, var: ir::Variable) {
    let mut last_block = start_block.clone();
    for block in start_block.linear_iter() {
        last_block = block.clone();

        let mut statements = block.get_statements();

        let first_use_index = statements
            .iter()
            .enumerate()
            .find(|(_, s)| s.used_vars().contains(&var))
            .map(|(i, _)| i);

        if let Some(index) = first_use_index {
            statements.insert(
                index,
                ir::Statement::Assignment {
                    target: var,
                    value: ir::Value::Unknown,
                },
            );
            block.set_statements(statements);

            return;
        }
    }

    let mut statements = last_block.get_statements();

    let index = statements
        .iter()
        .enumerate()
        .find(|(_, stmnt)| {
            matches!(
                stmnt,
                ir::Statement::Jump(_, _) | ir::Statement::JumpTrue(_, _, _)
            )
        })
        .map(|(i, _)| i);

    match index {
        Some(index) => {
            statements.insert(
                index,
                ir::Statement::Assignment {
                    target: var,
                    value: ir::Value::Unknown,
                },
            );
        }
        None => {
            todo!()
        }
    };
}

fn replace_in_cond_start(
    start_block: ir::BasicBlock,
    spilled: ir::Variable,
    replacement: ir::Variable,
    end_common_ptr: *const ir::InnerBlock,
) {
    let mut last_linear = start_block.clone();
    for block in start_block.linear_iter().skip(1) {
        assert_ne!(block.as_ptr(), end_common_ptr);

        let tmp = block.get_statements();
        block.set_statements(
            tmp.into_iter()
                .map(|s| replace::statement(s, &spilled, &replacement))
                .collect(),
        );

        last_linear = block;
    }

    let succs = last_linear.successors();
    dbg!(succs.len());

    if succs.len() == 1 {
        let (_, next_block) = succs.into_iter().next().unwrap();

        if next_block.as_ptr() == end_common_ptr {
            todo!("Reached the End-Block")
        }
    }

    todo!()
}

pub fn spill(
    start_block: ir::BasicBlock,
    start_index: usize,
    spilled: ir::Variable,
    replacement: ir::Variable,
    end_common: ir::BasicBlock,
) {
    let initial_statements = start_block.get_statements();
    let mut initial_iter = initial_statements.into_iter();

    let mut resulting: Vec<_> = initial_iter.by_ref().take(start_index).collect();
    resulting.push(ir::Statement::SaveVariable {
        var: spilled.clone(),
    });
    resulting.extend(initial_iter.map(|s| replace::statement(s, &spilled, &replacement)));
    start_block.set_statements(resulting);

    replace_in_cond_start(
        start_block.clone(),
        spilled.clone(),
        replacement.clone(),
        end_common.as_ptr(),
    );

    insert_load(start_block.clone(), replacement.clone());
}

pub fn spill_outer(
    header: ir::BasicBlock,
    end: ir::BasicBlock,
    variable: ir::Variable,
    replacement: ir::Variable,
) {
    // Insert the Save at the correct location
    let mut header_stmnts = header.get_statements();
    let header_save_index_res = header_stmnts
        .iter()
        .enumerate()
        .find(|(_, s)| {
            matches!(
                s,
                ir::Statement::Jump(_, _) | ir::Statement::JumpTrue(_, _, _)
            )
        })
        .map(|(i, _)| i);

    let header_save_index =
        header_save_index_res.expect("The Block needs to have at least oen jump");

    header_stmnts.insert(
        header_save_index,
        ir::Statement::SaveVariable { var: variable },
    );
    header.set_statements(header_stmnts);

    dbg!(end.as_ptr());

    // Insert the Load at the correct location
    let mut end_stmnts = end.get_statements();
    end_stmnts.insert(
        0,
        ir::Statement::Assignment {
            target: replacement,
            value: ir::Value::Unknown,
        },
    );
    end.set_statements(end_stmnts);
}
