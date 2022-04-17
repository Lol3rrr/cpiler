use std::collections::HashMap;

use isas::sh4a;

mod asm;

fn instruction_size(i: &sh4a::Instruction) -> u32 {
    match i {
        sh4a::Instruction::Nop
        | sh4a::Instruction::MovT { .. }
        | sh4a::Instruction::MovRR { .. }
        | sh4a::Instruction::MovIR { .. }
        | sh4a::Instruction::MovRPR { .. }
        | sh4a::Instruction::MovLR0PRR { .. }
        | sh4a::Instruction::MovLRR0PR { .. }
        | sh4a::Instruction::PushL { .. }
        | sh4a::Instruction::PushPR { .. }
        | sh4a::Instruction::PopL { .. }
        | sh4a::Instruction::PopPR { .. }
        | sh4a::Instruction::CmpGt { .. }
        | sh4a::Instruction::CmpPl { .. }
        | sh4a::Instruction::Sub { .. }
        | sh4a::Instruction::ShldRR { .. }
        | sh4a::Instruction::OrRR { .. } => 1,
        sh4a::Instruction::MovImmR { .. } => {
            // 1. Load of the Value
            // 2. Nop
            // 3. Jump over Value
            // 4. Nop
            // 5. Value
            // 6. Value
            6
        }
        sh4a::Instruction::JumpSubroutine { .. } => 1,
        sh4a::Instruction::JumpLabel { .. } => {
            // 1. The Load of the Address
            // 2. Nop as it would otherwise a slot illegal instruction
            // 3. The Branch itself
            // 4. The Nop needed after the Branch
            // (5.) Possible Nop to achieve correct alignment
            // 5(+1). The first Part of the Address of where to jump
            // 6(+1). The second Part of the Address of where to jump
            7
        }
        sh4a::Instruction::BranchTrueLabel { .. } => {
            // 1. Branch over the Jump if the condition is not true
            // 2. Nop needed after Branch
            // 3. Load the Target Address
            // 4. Nop
            // 5. Jump to the loaded Address
            // 6. Nop after the Jump
            // (7.) Possible Nop to achieve correct alignment
            // 7(+1). The Address to Jump to (Part 1)
            // 8(+1). The Address to Jump to (Part 2)
            9
        }
        sh4a::Instruction::Return => {
            // 1. The Return instruction itself
            // 2. Nop needed after Branch
            2
        }
        other => {
            dbg!(&other);
            todo!()
        }
    }
}

fn instruction_count(block: &sh4a::Block) -> u32 {
    block.instructions.iter().map(instruction_size).sum()
}

fn generate_block(block: sh4a::Block, offsets: &HashMap<String, u32>) -> (String, Vec<u8>) {
    let instruction_count = instruction_count(&block);

    let start_pc = *offsets.get(&block.name).unwrap();
    let asm_block = block.instructions.into_iter().fold(
        Vec::with_capacity(instruction_count as usize * 2),
        |mut instrs, instr| {
            let pc = start_pc + (instrs.len() * 2) as u32;
            instrs.extend(asm::Instruction::from_instr(instr, pc, offsets));

            instrs
        },
    );

    let result = asm_block
        .into_iter()
        .flat_map(|i| i.into_bytes().to_be_bytes())
        .collect();

    (block.name, result)
}

fn initial_block(main_block: String) -> sh4a::Block {
    let mut result = Vec::new();
    //result.extend(stack_instr);
    result.extend(vec![sh4a::Instruction::JumpLabel { label: main_block }]);

    sh4a::Block {
        name: "start".to_string(),
        instructions: result,
    }
}

pub fn assemble(main_block: String, blocks: Vec<sh4a::Block>) -> Vec<u8> {
    let initial_block = initial_block(main_block);

    let block_lengths = std::iter::once(initial_block.clone())
        .chain(blocks.iter().cloned())
        .map(|b| (b.name.clone(), instruction_count(&b)));

    // TODO
    // Add a start block that will be at address 0 and will just jump to the main block

    let block_offsets = {
        let mut address = 0;
        let mut tmp: HashMap<String, u32> = HashMap::new();

        for (name, instruction_count) in block_lengths {
            tmp.insert(name, address);
            address += instruction_count * 2;

            if address % 4 != 0 {
                address += 4 - (address % 4);
            }
        }

        tmp
    };

    let final_blocks: Vec<(u32, Vec<u8>)> = std::iter::once(initial_block)
        .chain(blocks.into_iter())
        .map(|b| generate_block(b, &block_offsets))
        .map(|(name, data)| {
            let offset = *block_offsets.get(&name).unwrap();
            (offset, data)
        })
        .collect();

    let mut result = Vec::new();

    for (start, data) in final_blocks {
        let data_size = data.len();
        let data_start = start as usize;
        let data_end = data_start + data_size;

        if result.len() < data_end {
            result.resize_with(data_end, || 0);
        }

        let result_space = result.get_mut(data_start..data_end).unwrap();

        result_space.clone_from_slice(&data);
    }

    result
}
