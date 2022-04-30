use std::collections::HashMap;
use std::sync::{atomic, Arc};

use general::arch::Arch;
use ir::BasicBlock;

#[derive(Debug)]
pub struct ConvertContext {
    arch: Arch,
    loop_ctx: Option<(BasicBlock, BasicBlock)>,
    current_tmp: Arc<atomic::AtomicUsize>,
    global: bool,
    global_vars: HashMap<String, ir::Variable>,
}

impl ConvertContext {
    pub fn base(arch: Arch, globals: HashMap<String, ir::Variable>) -> Self {
        Self {
            arch,
            loop_ctx: None,
            current_tmp: Arc::new(atomic::AtomicUsize::new(0)),
            global: false,
            global_vars: globals,
        }
    }

    pub fn set_global(&mut self, n_value: bool) {
        self.global = n_value;
    }

    pub fn next_tmp(&self) -> usize {
        self.current_tmp.fetch_add(1, atomic::Ordering::SeqCst)
    }

    pub fn get_loop_start(&self) -> Option<&BasicBlock> {
        let (start, _) = self.loop_ctx.as_ref()?;
        Some(start)
    }
    pub fn get_loop_end(&self) -> Option<&BasicBlock> {
        let (_, end) = self.loop_ctx.as_ref()?;
        Some(end)
    }

    pub fn global(&self) -> bool {
        self.global
    }

    pub fn arch(&self) -> &Arch {
        &self.arch
    }
    pub fn pointer_size(&self) -> usize {
        self.arch.ptr_size()
    }
    pub fn pointer_alignment(&self) -> usize {
        self.arch.ptr_size()
    }
    pub fn pointer_type(&self) -> ir::Type {
        match self.arch.ptr_size() {
            1 => ir::Type::I8,
            2 => ir::Type::I16,
            4 => ir::Type::I32,
            8 => ir::Type::I64,
            unexpected => panic!("Unexpected Ptr Size: {:?}", unexpected),
        }
    }
    pub fn pointer_constant(&self, value: u64) -> ir::Constant {
        match self.arch.ptr_size() {
            1 => ir::Constant::U8(value as u8),
            2 => ir::Constant::U16(value as u16),
            4 => ir::Constant::U32(value as u32),
            8 => ir::Constant::U64(value as u64),
            _ => panic!("Unexpected Size"),
        }
    }

    pub fn with_loop(&self, start: BasicBlock, end: BasicBlock) -> Self {
        Self {
            arch: self.arch.clone(),
            loop_ctx: Some((start, end)),
            current_tmp: self.current_tmp.clone(),
            global: self.global,
            global_vars: self.global_vars.clone(),
        }
    }

    pub fn get_global(&self, name: &str) -> Option<&ir::Variable> {
        self.global_vars.get(name)
    }
}
