use crate::{AExpression, AScope, AStatement};

fn insert_updates(scope: &mut AScope, updates: &[AStatement]) {
    let start = 0;
    loop {
        let sub_set = &scope.statements[start..];

        let raw_found = sub_set
            .iter()
            .enumerate()
            .find(|(_, s)| matches!(s, AStatement::Continue))
            .map(|(i, _)| i);

        match raw_found {
            Some(found) => {
                todo!()
            }
            None => break,
        };
    }
}

pub fn convert(condition: AExpression, mut body: AScope, updates: Vec<AStatement>) -> AStatement {
    insert_updates(&mut body, &updates);

    body.statements.extend(updates);

    AStatement::WhileLoop { condition, body }
}
