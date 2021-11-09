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
    Op(O),
    Value(V),
}

enum Intermediate<O, V> {
    Value(V),
    Operator(O),
}

pub fn parse<I, II, O>(inputs: II) -> Result<O::Value, ()>
where
    II: IntoIterator<IntoIter = I, Item = Inputs<O, O::Value>>,
    I: Iterator<Item = Inputs<O, O::Value>>,
    O: Operator,
{
    let input = inputs.into_iter();

    let mut output = Vec::new();
    let mut op_stack: Vec<O> = Vec::new();

    for tmp in input {
        match tmp {
            Inputs::Op(op) => {
                let c_prio = op.priority();

                while let Some(latest_op) = op_stack.last() {
                    let last_prio = latest_op.priority();
                    let last_asso = latest_op.assosication();

                    if last_prio > c_prio
                        || (last_prio == c_prio && last_asso == OpAssosication::Left)
                    {
                        let latest = op_stack.pop().unwrap();
                        output.push(Intermediate::Operator(latest));

                        continue;
                    }

                    break;
                }

                op_stack.push(op);
            }
            Inputs::Value(value) => {
                output.push(Intermediate::Value(value));
            }
        }
    }

    for op in op_stack {
        output.push(Intermediate::Operator(op));
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

    result.pop().ok_or(())
}
