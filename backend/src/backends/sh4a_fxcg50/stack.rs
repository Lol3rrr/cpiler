use std::collections::{HashMap, HashSet};

use crate::isas::sh4a;

#[derive(Debug)]
pub struct StackManagment {
    pub setup: Vec<sh4a::Instruction>,
    pub teardown: Vec<sh4a::Instruction>,
    pub offsets: HashMap<ir::Variable, isize>,
}

struct StackAssign<AI> {
    offset: usize,
    allocs: AI,
}

impl<AI> StackAssign<AI> {
    pub fn new(iter: AI) -> Self {
        Self {
            offset: 0,
            allocs: iter,
        }
    }

    pub fn get_offset(&self) -> usize {
        self.offset
    }
}

impl<AI> Iterator for StackAssign<AI>
where
    AI: Iterator<Item = (usize, usize)>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let (align, size) = self.allocs.next()?;

        if self.offset % align != 0 {
            self.offset += align - (self.offset % align);
        }

        let result = self.offset;
        self.offset += size;

        Some(result)
    }
}

pub fn setup<RI>(func: &ir::FunctionDefinition, registers: RI)
where
    RI: Iterator<Item = sh4a::Register>,
{
    dbg!(&func);

    let registers: Vec<_> = {
        let tmp: HashSet<_> = registers.collect();
        tmp.into_iter().collect()
    };
    dbg!(&registers);

    let reg_iter = registers
        .iter()
        .cloned()
        .chain(std::iter::once(sh4a::Register::PR));

    let alloc_iter = reg_iter.clone().map(|r| match r {
        sh4a::Register::GeneralPurpose(_) => (4, 4),
        sh4a::Register::FloatingPoint(_) => todo!("Size + Alignment of FloatingPoint"),
        sh4a::Register::PR => (4, 4),
    });

    let mut stack_allocator = StackAssign::new(alloc_iter);

    let (stores, loads): (Vec<_>, Vec<_>) = reg_iter
        .clone()
        .zip(stack_allocator.by_ref())
        .map(|(reg, offset)| {
            dbg!(&reg, &offset);
            ((), ())
        })
        .unzip();

    dbg!(&stores, &loads);

    let entire_space = stack_allocator.get_offset();
    dbg!(&entire_space);

    todo!()
}
