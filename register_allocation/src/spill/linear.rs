use crate::context::linear_spill;

use super::replace;

fn insert_load(start_block: ir::BasicBlock, var: ir::Variable) {
    let mut last_block = start_block.clone();

    for block in start_block.block_iter() {
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
            dbg!(&block);

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
                ir::Statement::Jump(_) | ir::Statement::JumpTrue(_, _)
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
    last_block.set_statements(statements);
}

pub fn spill(
    start_block: ir::BasicBlock,
    spill: linear_spill::SpillResult,
    replacement: ir::Variable,
) {
    let initial_statements = start_block.get_statements();
    let mut initial_iter = initial_statements.into_iter();

    let mut resulting: Vec<_> = initial_iter.by_ref().take(spill.save_index).collect();
    resulting.push(ir::Statement::SaveVariable {
        var: spill.var.clone(),
    });
    resulting.extend(
        initial_iter.map(|s| replace::statement(s, &spill.var, &replacement, &spill.load_block)),
    );
    start_block.set_statements(resulting);

    for block in start_block.block_iter().skip(1) {
        let tmp = block.get_statements();
        block.set_statements(
            tmp.into_iter()
                .map(|s| replace::statement(s, &spill.var, &replacement, &spill.load_block))
                .collect(),
        );
    }

    dbg!(
        &spill.var,
        &spill.save_index,
        &spill.load_block,
        &spill.load_index
    );

    /*
    let load_index = if spill.load_block.as_ptr() == start_block.as_ptr() {
        spill.load_index + 1
    } else {
        spill.load_index
    };
    let mut load_stmnts = spill.load_block.get_statements();
    load_stmnts.insert(
        load_index,
        ir::Statement::Assignment {
            target: replacement,
            value: ir::Value::Unknown,
        },
    );
    spill.load_block.set_statements(load_stmnts);
    */

    //todo!();
    insert_load(spill.load_block, replacement);
}
