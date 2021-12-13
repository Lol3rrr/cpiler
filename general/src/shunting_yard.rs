use std::fmt::Debug;

pub trait Operator {
    type Value;

    fn assosication(&self) -> OpAssosication;
    fn priority(&self) -> usize;

    fn operands(&self) -> usize;

    fn to_value(self, operands: Vec<Self::Value>) -> Self::Value;
}

#[derive(Debug, PartialEq)]
pub enum OpAssosication {
    Left,
    Right,
}

#[derive(Debug, PartialEq)]
pub enum Inputs<O, V> {
    Op(OpItem<O>),
    Value(V),
}

#[derive(Debug)]
enum Intermediate<O, V> {
    Value(V),
    Operator(O),
}

#[derive(Debug, PartialEq)]
pub enum OpItem<O> {
    OpenParen,
    CloseParen,
    Op(O),
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    EmptyResult,
}

pub fn parse<I, II, O>(inputs: II) -> Result<O::Value, ParseError>
where
    II: IntoIterator<IntoIter = I, Item = Inputs<O, O::Value>>,
    I: Iterator<Item = Inputs<O, O::Value>>,
    O: Operator + Debug,
    O::Value: Debug,
{
    let input = inputs.into_iter();

    let mut output: Vec<Intermediate<O, O::Value>> = Vec::new();
    let mut op_stack: Vec<OpItem<O>> = Vec::new();

    for tmp in input {
        match tmp {
            Inputs::Op(op_item) => {
                match op_item {
                    OpItem::OpenParen => {
                        op_stack.push(OpItem::OpenParen);
                    }
                    OpItem::CloseParen => {
                        while let Some(op_item) = op_stack.pop() {
                            match op_item {
                                OpItem::Op(op) => {
                                    dbg!(&op);
                                    output.push(Intermediate::Operator(op));
                                }
                                OpItem::OpenParen => break,
                                OpItem::CloseParen => unreachable!(
                                    "There should never be a Closing Paren on the Op-Stack"
                                ),
                            };
                        }
                    }
                    OpItem::Op(op) => {
                        let c_prio = op.priority();
                        let c_asso = op.assosication();

                        while let Some(latest_op_item) = op_stack.pop() {
                            match latest_op_item {
                                OpItem::OpenParen => {
                                    op_stack.push(OpItem::OpenParen);
                                }
                                OpItem::CloseParen => unreachable!(
                                    "There should never be a Closing Paren on the Op-Stack"
                                ),
                                OpItem::Op(latest_op) => {
                                    let last_prio = latest_op.priority();

                                    if last_prio > c_prio
                                        || (last_prio == c_prio && c_asso == OpAssosication::Left)
                                    {
                                        output.push(Intermediate::Operator(latest_op));

                                        continue;
                                    } else {
                                        op_stack.push(OpItem::Op(latest_op));
                                    }

                                    break;
                                }
                            };
                        }

                        op_stack.push(OpItem::Op(op));
                    }
                };
            }
            Inputs::Value(value) => {
                output.push(Intermediate::Value(value));
            }
        }
    }

    while let Some(op_item) = op_stack.pop() {
        match op_item {
            OpItem::Op(op) => {
                output.push(Intermediate::Operator(op));
            }
            other => {
                dbg!(&other);
                panic!("Unexpected Op-Item")
            }
        };
    }

    let mut result: Vec<O::Value> = Vec::new();

    for entry in output {
        match entry {
            Intermediate::Value(v) => {
                result.push(v);
            }
            Intermediate::Operator(op) => {
                let operand_count = op.operands();
                let mut operands = Vec::with_capacity(operand_count);

                for _ in 0..operand_count {
                    let tmp = result.pop().unwrap();
                    operands.push(tmp);
                }

                let n_value = op.to_value(operands);
                result.push(n_value);
            }
        };
    }

    result.pop().ok_or(ParseError::EmptyResult)
}
