use crate::spill;

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
        .find(|(_, s)| matches!(s, ir::Statement::Jump(_) | ir::Statement::JumpTrue(_, _)))
        .map(|(i, _)| i);

    let header_save_index =
        header_save_index_res.expect("The Block needs to have at least oen jump");

    header_stmnts.insert(
        header_save_index,
        ir::Statement::SaveVariable { var: variable },
    );
    header.set_statements(header_stmnts);

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

pub fn spill_inner(
    save_block: ir::BasicBlock,
    save_index: usize,
    load_block: ir::BasicBlock,
    load_index: usize,
    variable: ir::Variable,
    replacement: ir::Variable,
) {
    let mut save_statements = save_block.get_statements();
    save_statements.insert(save_index, spill::save_statement(variable));
    save_block.set_statements(save_statements);

    let mut load_statements = load_block.get_statements();
    let load_stmnt = spill::load_statement(replacement);
    if save_block.as_ptr() == load_block.as_ptr() {
        load_statements.insert(load_index + 1, load_stmnt);
    } else {
        load_statements.insert(load_index, load_stmnt);
    }
    load_block.set_statements(load_statements);
}
