//! The Textual representation for the IR

use crate::{Expression, FunctionDefinition, Operand, Statement, Value, Variable};

pub fn generate_text_rep(func: &FunctionDefinition) -> String {
    let func_header = format!("fn {}() -> {:?}", func.name, func.return_ty);

    let block_content = func
        .block
        .block_iter()
        .map(|b| {
            let b_header = format!("  block 0x{:x}", b.as_ptr() as usize);
            let b_content = b
                .get_statements()
                .into_iter()
                .map(|s| format!("    {}", statement_content(&s)))
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}\n{}", b_header, b_content)
        })
        .collect::<Vec<_>>()
        .join("\n\n");

    format!("{}\n{}", func_header, block_content)
}

fn statement_content(stmnt: &Statement) -> String {
    match stmnt {
        Statement::SaveVariable { var } => format!("Save-Variable {}", variable_content(var)),
        Statement::Assignment { target, value } => {
            format!("{} = {}", variable_content(target), value_content(value))
        }
        Statement::Jump(target, _) => format!("Jump 0x{:x}", target.as_ptr() as usize),
        Statement::JumpTrue(cond, target, _) => format!(
            "JumpTrue 0x{:x} if {}",
            target.as_ptr() as usize,
            variable_content(cond)
        ),
        Statement::Return(Some(ret_var)) => format!("Return {}", variable_content(ret_var)),
        Statement::Return(None) => "Return".to_string(),
        other => format!("{:?}", other),
    }
}

fn operand_content(oper: &Operand) -> String {
    match oper {
        Operand::Variable(var) => variable_content(var),
        Operand::Constant(con) => format!("{:?}", con),
    }
}

fn variable_content(var: &Variable) -> String {
    format!("{}@{}({:?})", var.name, var.generation(), var.ty)
}

fn value_content(value: &Value) -> String {
    match value {
        Value::Expression(exp) => expression_content(exp),
        Value::Phi { sources } => format!(
            "Phi ({})",
            sources
                .iter()
                .map(|s| format!(
                    "{} from 0x{:x}",
                    variable_content(&s.var),
                    s.block.as_ptr() as usize,
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        other => format!("{:?}", other),
    }
}

fn expression_content(exp: &Expression) -> String {
    match exp {
        Expression::Cast { base, target } => format!("({:?}) {}", target, operand_content(base)),
        Expression::BinaryOp { op, left, right } => format!(
            "{} {:?} {}",
            operand_content(left),
            op,
            operand_content(right)
        ),
        other => format!("{:?}", other),
    }
}
