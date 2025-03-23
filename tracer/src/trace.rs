use std::cell::RefCell;

use common::rv_trace::{ELFInstruction, MemoryState, RVTraceRow, RegisterState};


// TODO(Maks) try remove RefCell
pub struct Tracer<const XLEN: u8> {
    pub rows: RefCell<Vec<RVTraceRow<XLEN>>>,
    open: RefCell<bool>,
}

impl<const XLEN: u8> Tracer<XLEN> {
    pub fn new() -> Self {
        Self {
            rows: RefCell::new(Vec::new()),
            open: RefCell::new(false),
        }
    }

    pub fn start_instruction(&self, inst: ELFInstruction<XLEN>) {
        let mut inst = inst;
        // TODO(Maks) make const
        if XLEN == 32 {
            inst.address = inst.address as u32 as u64;
        }
        
        *self.open.try_borrow_mut().unwrap() = true;
        self.rows.try_borrow_mut().unwrap().push(RVTraceRow {
            instruction: inst,
            register_state: RegisterState::default(),
            memory_state: None,
            advice_value: None,
            precompile_input: None,
            precompile_output_address: None,
        });
    }

    pub fn capture_pre_state(&self, reg: [i64; 32]) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }

        let mut rows = self.rows.try_borrow_mut().unwrap();
        let row = rows.last_mut().unwrap();

        if let Some(rs1) = row.instruction.rs1 {
            row.register_state.rs1_val = Some(normalize_register_value::<XLEN>(reg[rs1 as usize]));
        }

        if let Some(rs2) = row.instruction.rs2 {
            row.register_state.rs2_val = Some(normalize_register_value::<XLEN>(reg[rs2 as usize]));
        }
    }

    pub fn capture_post_state(&self, reg: [i64; 32]) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }

        let mut rows = self.rows.try_borrow_mut().unwrap();
        let row = rows.last_mut().unwrap();

        if let Some(rd) = row.instruction.rd {
            row.register_state.rd_post_val = Some(normalize_register_value::<XLEN>(reg[rd as usize]));
        }
    }

    pub fn push_memory(&self, memory_state: MemoryState) {
        if !*self.open.try_borrow().unwrap() {
            return;
        }

        if let Some(row) = self.rows.try_borrow_mut().unwrap().last_mut() {
            row.memory_state = Some(memory_state);
        }
    }

    pub fn end_instruction(&self) {
        *self.open.try_borrow_mut().unwrap() = false;
    }
}

// TODO(Maks) make const
fn normalize_register_value<const XLEN: u8>(value: i64) -> u64 {
    match XLEN {
        32 => value as u32 as u64,
        64 => value as u64,
        _ => panic!("incorrect XLEN")
    }
}
