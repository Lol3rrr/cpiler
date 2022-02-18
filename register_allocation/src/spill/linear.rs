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
        None => {}
    };
    last_block.set_statements(statements);
}

fn insert_save(
    start_block: &ir::BasicBlock,
    spill: &linear_spill::SpillResult,
) -> Option<ir::Statement> {
    if spill.save_index == 0 {
        let initial_statements = start_block.get_statements();
        let spilled_stmnt = initial_statements.get(spill.save_index).unwrap();

        match spilled_stmnt {
            ir::Statement::Assignment {
                value: ir::Value::Phi { sources },
                ..
            } => {
                for src in sources {
                    let src_block = src.block.upgrade().unwrap();

                    let mut src_statements =
                        src_block.get_statements().into_iter().enumerate().rev();
                    let (index, _) = src_statements
                        .find(|(_, stmnt)| match stmnt {
                            ir::Statement::Jump(j_target) => {
                                j_target.as_ptr() == start_block.as_ptr()
                            }
                            ir::Statement::JumpTrue(_, j_target) => {
                                j_target.as_ptr() == start_block.as_ptr()
                            }
                            _ => false,
                        })
                        .unwrap();

                    let mut stmnts = src_block.get_statements();
                    stmnts.insert(
                        index,
                        ir::Statement::SaveVariable {
                            var: spill.var.clone(),
                        },
                    );
                    src_block.set_statements(stmnts);
                }

                return None;
            }
            _ => {}
        };
    }

    Some(ir::Statement::SaveVariable {
        var: spill.var.clone(),
    })
}

pub fn spill(
    start_block: ir::BasicBlock,
    spill: linear_spill::SpillResult,
    replacement: ir::Variable,
) {
    let initial_statements = start_block.get_statements();
    let mut initial_iter = initial_statements.into_iter();
    let mut resulting: Vec<_> = initial_iter.by_ref().take(spill.save_index).collect();
    if let Some(save_stmnt) = insert_save(&start_block, &spill) {
        resulting.push(save_stmnt);
    }
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

    insert_load(spill.load_block, replacement);
}
