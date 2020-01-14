use crate::code_generator::intermediate::{Context, Instruction, Access, Variable, UniqueVariable, VariableIndex, Constant, Label};
use ::virtual_machine::instruction::Instruction as VmInstruction;
use parser::ast::ExprOp;
use std::collections::BTreeMap;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MemoryLocation(pub u64);

impl std::ops::Add<u64> for MemoryLocation {
    type Output = MemoryLocation;

    fn add(self, rhs: u64) -> Self::Output {
        MemoryLocation(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u64> for MemoryLocation {
    fn add_assign(&mut self, rhs: u64) {
        self.0 += rhs
    }
}

type MemoryStorage = BTreeMap<VariableIndex, (MemoryLocation, Option<i64>)>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MemoryRange(MemoryLocation, MemoryLocation);

impl MemoryRange {
    pub fn new(start: u64, end: u64) -> Self {
        MemoryRange(MemoryLocation(start), MemoryLocation(end))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Segments {
    arrays: Option<MemoryRange>,
    variables: Option<MemoryRange>,
    temporaries: Option<MemoryRange>,
}

#[derive(Debug)]
struct Memory {
    storage: MemoryStorage,
    segments: Segments,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            storage: MemoryStorage::new(),
            segments: Segments {
                arrays: None,
                variables: None,
                temporaries: None,
            },
        }
    }

    fn add_variable(&mut self, index: VariableIndex, value: Option<i64>) {
        let last = if let Some(MemoryRange(_, ref mut end)) = self.segments.variables {
            *end += 1;
            *end
        } else {
            let last = self.segments.arrays.as_ref().map_or(MemoryLocation(0), |s| s.1) + 1;
            self.segments.variables = Some(MemoryRange(last, last));
            last
        };

        self.storage.insert(index, (last, value));
    }

    fn get_location(&self, index: VariableIndex) -> MemoryLocation {
        self.storage[&index].0
    }
}


fn compare_variables(a: &UniqueVariable, b: &UniqueVariable) -> Ordering {
    match (a.variable(), b.variable()) {
        (v1@Variable::Array { .. }, v2@Variable::Array { .. }) => v1.size().cmp(&v2.size()),
        (Variable::Unit { .. }, Variable::Unit { .. }) => Ordering::Equal,
        (Variable::Array { .. }, Variable::Unit { .. }) => Ordering::Less,
        (Variable::Unit { .. }, Variable::Array { .. }) => Ordering::Greater,
    }
}

struct InstructionManager {
    target_instructions: Vec<VmInstruction>,
    label_positions: BTreeMap<Label, u64>,
    back_patches_list: BTreeMap<Label, Vec<usize>>,
}

impl InstructionManager {
    fn fix_label(&mut self, instruction_ptr: usize, target_pointer: u64) {
        match self.target_instructions[instruction_ptr] {
            VmInstruction::Jump(ref mut target)
            | VmInstruction::Jpos(ref mut target)
            | VmInstruction::Jneg(ref mut target)
            | VmInstruction::Jzero(ref mut target) => {
                *target = target_pointer;
            },
            _ => unreachable!(),
        }
    }

    fn translate_jump<F: FnOnce(u64) -> VmInstruction>(&mut self, label: &Label, create: F) {
        if let Some(pos) = self.label_positions.get(label) {
            self.target_instructions.push(create(*pos));
        } else {
            let pos = self.target_instructions.len();
            self.back_patches_list.entry(*label).or_default().push(pos);
            self.target_instructions.push(create(u64::max_value()));
        }
    }

    fn translate_label(&mut self, label: &Label) {
        let target = self.target_instructions.len() as u64;
        self.label_positions.insert(*label, target);
        if let Some(backlist) = self.back_patches_list.remove(&label) {
            for pos in backlist {
                self.fix_label(pos, target);
            }
        }
    }
}

pub struct Generator {
    context: Context,
    memory: Memory,
    instruction_manager: InstructionManager,
}

impl Generator {
    pub fn new(context: Context) -> Self {
        let cap = context.instructions().len() * 4;
        Generator {
            context,
            memory: Memory::new(),
            instruction_manager: InstructionManager {
                target_instructions: Vec::with_capacity(cap),
                label_positions: BTreeMap::new(),
                back_patches_list: BTreeMap::new(),
            },
        }
    }

    fn allocate_memory(&mut self) {
        if self.context.variables().is_empty() {
            return;
        }

        let mut variables: Vec<_> = self.context.variables().iter().map(|v| v).collect();
        variables.sort_unstable_by(|&a, &b| compare_variables(a, b));

        let middle = variables.binary_search_by(|&a| {
            match a.variable() {
                Variable::Array { .. } => Ordering::Less,
                Variable::Unit { .. } => Ordering::Greater,
            }
        }).expect_err("incorrect ordering function");

        let iter = if middle > 0 {
            let arrays_segment_end: usize = variables.iter().take(middle).map(|&arr| arr.variable().size()).sum();
            self.memory.segments.arrays = Some(MemoryRange(MemoryLocation(1), MemoryLocation(arrays_segment_end as u64)));

            let mut iter = variables.iter();
            let arrays = iter.by_ref().take(middle);

            let array_base_indexes = arrays.scan(1, |first, &arr| {
                let start_index = *first;
                *first += arr.variable().size();
                if let Variable::Array { start, .. } = arr.variable() {
                    Some((arr, start_index as i64 - *start))
                } else {
                    panic!("incorrect variable order");
                }
            });

            for (ind, (arr, base_index)) in array_base_indexes.enumerate() {
                self.memory.add_variable(arr.id(), Some(base_index));
            }

            iter
        } else {
            variables.iter()
        };

        for (ind, &var) in iter.enumerate() {
            self.memory.add_variable(var.id(), None);
        }
    }

    fn get_constant_location(&self, value: i64) -> MemoryLocation {
        let ind = self.context.get_constant_index(&Constant(value));
        self.memory.storage.get(&ind).expect(format!("constant {} has not been generated", value).as_str()).0
    }

    fn generate_constant(&mut self, value: i64, location: MemoryLocation) {
        let abs = value.abs() as u64;
        if abs < 10 {
            let (grow_instr, shrink_instr) = if value.is_positive() {
                (VmInstruction::Inc, VmInstruction::Dec)
            } else {
                (VmInstruction::Dec, VmInstruction::Inc)
            };

            for _ in 0..value.abs() {
                self.instruction_manager.target_instructions.push(grow_instr);
            }
            self.instruction_manager.target_instructions.push(VmInstruction::Store(location.0 as u64));
            for _ in 0..value.abs() {
                self.instruction_manager.target_instructions.push(shrink_instr);
            }
        } else {
            let leading_zeros = abs.leading_zeros();
            let mut abs = abs.reverse_bits();

            let one_const = self.get_constant_location(1);
            let two_const = self.get_constant_location(2);

            let grow_instr = if value.is_positive() {
                VmInstruction::Inc
            } else {
                VmInstruction::Dec
            };


            while abs & 1 == 0 {
                abs >>= 1;
            }

            for i in 0..(64 - leading_zeros - 1) {
                if abs & 1 == 1 {
                    self.instruction_manager.target_instructions.push(grow_instr);
                }

                self.instruction_manager.target_instructions.push(VmInstruction::Shift(one_const.0));
                abs >>= 1;
            }
            if abs & 1 == 1 {
                self.instruction_manager.target_instructions.push(grow_instr);
            }

            self.instruction_manager.target_instructions.push(VmInstruction::Store(location.0));
            self.instruction_manager.target_instructions.push(VmInstruction::Sub(0));
        }
    }

    fn generate_constants(&mut self) {
        for (constant, index) in self.context.constants() {
            let value = self.memory.storage.get_mut(index).expect("constant not in memory");
            value.1 = Some(constant.value());
        }

        let mut to_generate: Vec<_> = self.memory.storage.iter().filter_map(|(_, &(loc, val))| {
            val.map(|val| (loc, val))
        }).collect();

        to_generate.sort_unstable_by(|(_, val1), (_, val2)| {
            let cmp = val1.abs().cmp(&val2.abs());
            if cmp == Ordering::Equal {
                val1.cmp(val2)
            } else {
                cmp
            }
        });

        println!("generating constants: {:?}", to_generate);

        self.instruction_manager.target_instructions.push(VmInstruction::Sub(0));

        for (loc, val) in to_generate {
            self.generate_constant(val, loc);
        }
    }

    pub fn translate(mut self) -> Vec<VmInstruction> {
        let simple_constants = vec![
            Constant(0),
            Constant(1),
            Constant(-1),
            Constant(2),
            Constant(-2),
        ];

        for c in &simple_constants {
            self.context.register_constant(c.clone());
        }

        self.context.add_variable(Variable::Unit { name: String::from("tmp$mul_left") });
        self.context.add_variable(Variable::Unit { name: String::from("tmp$result") });

        self.allocate_memory();
        self.generate_constants();

        println!("{:?}", self.context.variables());
        println!("{:?}", self.memory.storage);

        let ir_instructions = self.context.instructions().to_vec();

        for instruction in &ir_instructions {
            match instruction {
                Instruction::Label { label } => {
                    self.instruction_manager.translate_label(label);
                },
                Instruction::Load { access } => {
                    match access {
                        Access::Constant(c) => {
                            let ind = self.context.get_constant_index(c);
                            let loc = self.memory.get_location(ind);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(loc.0));
                        },
                        Access::Variable(ind) => {
                            let loc = self.memory.get_location(*ind);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(loc.0));
                        },
                        Access::ArrayStatic(arr, c) => {
                            let real_arr_loc = self.memory.storage[arr].1.expect("unallocated array");
                            self.instruction_manager.target_instructions.push(VmInstruction::Load((real_arr_loc + c.value()) as u64));
                        },
                        Access::ArrayDynamic(arr, ind) => {
                            let arr_loc = self.memory.get_location(*arr);
                            let ind_loc = self.memory.get_location(*ind);

                            self.instruction_manager.target_instructions.push(VmInstruction::Load(arr_loc.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Add(ind_loc.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Loadi(0));
                        },
                    }
                },
                Instruction::PreStore { access } => {
                    match access {
                        Access::Constant(_)
                        | Access::Variable(_)
                        | Access::ArrayStatic(_, _) => (),
                        Access::ArrayDynamic(_, _) => (),// unimplemented!(),
                    }
                },
                Instruction::Store { access } => {
                    match access {
                        Access::Constant(_) => panic!("can't store into a constant"),
                        Access::Variable(ind) => {
                            let loc = self.memory.get_location(*ind);
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(loc.0));
                        },
                        Access::ArrayStatic(arr, c) => {
                            let loc = self.memory.storage[arr].1.expect("unallocated array");
                            self.instruction_manager.target_instructions.push(VmInstruction::Store((loc + c.value()) as u64));
                        },
                        Access::ArrayDynamic(arr, ind) => {
                            let tmp = self.context.find_variable_by_name("tmp$1").expect("tmp$1 unavailable");
                            let tmp_loc = self.memory.get_location(tmp.id());
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(tmp_loc.0));

                            let arr_loc = self.memory.get_location(*arr);
                            let ind_loc = self.memory.get_location(*ind);

                            self.instruction_manager.target_instructions.push(VmInstruction::Load(arr_loc.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Add(ind_loc.0));

                            let tmp2 = self.context.find_variable_by_name("tmp$2").expect("tmp$2 unavailable");
                            let tmp2_loc = self.memory.get_location(tmp2.id());
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(tmp2_loc.0));

                            self.instruction_manager.target_instructions.push(VmInstruction::Load(tmp_loc.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Storei(tmp2_loc.0));
                        },
                    }
                },
                Instruction::Operation { op, operand } => {
                    let operand = self.memory.get_location(*operand);
                    match op {
                        ExprOp::Plus => self.instruction_manager.target_instructions.push(VmInstruction::Add(operand.0)),
                        ExprOp::Minus => self.instruction_manager.target_instructions.push(VmInstruction::Sub(operand.0)),
                        ExprOp::Times => {
                            // if b == 0 { goto end }
                            // if b < 0 {
                            //   b = -b
                            //   a = -a
                            // }
                            // result = 0
                            // while b > 0 {
                            //   if lsb(b) == 1 {
                            //     result += a
                            //   }
                            //   b >>= 1
                            //   a <<= 1
                            // }
                            let left = self.context.find_variable_by_name("tmp$mul_left").expect("tmp$mul_left unavailable").id();
                            let left = self.memory.get_location(left);
                            let right_tmp = self.context.find_variable_by_name("tmp$op").expect("tmp$op unavailable").id();
                            let right_tmp = self.memory.get_location(right_tmp);
                            let tmp = self.context.find_variable_by_name("tmp$1").expect("tmp$1 unavailable").id();
                            let tmp = self.memory.get_location(tmp);
                            let result = self.context.find_variable_by_name("tmp$result").expect("tmp$result unavailable").id();
                            let result = self.memory.get_location(result);
                            let const_1 = self.get_constant_location(1);
                            let const_neg_1 = self.get_constant_location(-1);

                            let label_start = self.context.new_label();
                            let label_main = self.context.new_label();
                            let label_step = self.context.new_label();
                            let label_end = self.context.new_label();
                            let label_real_end = self.context.new_label();

                            self.instruction_manager.target_instructions.push(VmInstruction::Store(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(operand.0));
//                            let label_end_backpatch1 = self.instruction_manager.target_instructions.len();
                            self.instruction_manager.translate_jump(&label_real_end, VmInstruction::Jzero);
//                            self.instruction_manager.target_instructions.push(VmInstruction::Jzero(u64::max_value())); //label end
//                            let label_start_backpatch = self.instruction_manager.target_instructions.len();
                            self.instruction_manager.translate_jump(&label_start, VmInstruction::Jpos);
//                            self.instruction_manager.target_instructions.push(VmInstruction::Jpos(u64::max_value())); //label start
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(operand.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(operand.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(right_tmp.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(right_tmp.0));
                            // label start
//                            let label_start_pos = self.instruction_manager.target_instructions.len() as u64;
//                            label_positions.insert(&label_start, label_start_pos);
//                            self.fix_label(label_start_backpatch, label_start_pos);
                            self.instruction_manager.translate_label(&label_start);
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(result.0));
                            // label main
//                            let label_main_pos = self.instruction_manager.target_instructions.len() as u64;
//                            label_positions.insert(&label_main, label_main_pos);
                            self.instruction_manager.translate_label(&label_main);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(right_tmp.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(tmp.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Shift(const_neg_1.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Shift(const_1.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Sub(tmp.0));
//                            let label_step_backpatch = self.instruction_manager.target_instructions.len();
//                            self.instruction_manager.target_instructions.push(VmInstruction::Jzero(u64::max_value())); //label step
                            self.instruction_manager.translate_jump(&label_step, VmInstruction::Jzero);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Add(result.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(result.0));
                            // label step
//                            let label_step_pos = self.instruction_manager.target_instructions.len() as u64;
//                            label_positions.insert(&label_step, label_step_pos);
//                            self.fix_label(label_step_backpatch, label_step_pos);
                            self.instruction_manager.translate_label(&label_step);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(right_tmp.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Shift(const_neg_1.0));
//                            self.instruction_manager.target_instructions.push(VmInstruction::Jzero(label_end));
                            self.instruction_manager.translate_jump(&label_end, VmInstruction::Jzero);
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(right_tmp.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(left.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Shift(const_1.0));
                            self.instruction_manager.target_instructions.push(VmInstruction::Store(left.0));
//                            self.instruction_manager.target_instructions.push(VmInstruction::Jump(label_main_pos));
                            self.instruction_manager.translate_jump(&label_main, VmInstruction::Jump);
                            // label end
                            self.instruction_manager.translate_label(&label_end);
                            self.instruction_manager.target_instructions.push(VmInstruction::Load(result.0));
                            self.instruction_manager.translate_label(&label_real_end);
                            // self.instruction_manager.target_instructions.push(VmInstruction::Mul(loc.0));
                            // unimplemented!("times operator")
                        },
                        ExprOp::Div => {
                            self.instruction_manager.target_instructions.push(VmInstruction::Div(operand.0));
                            // unimplemented!("div operator")
                        },
                        ExprOp::Mod => {
                            self.instruction_manager.target_instructions.push(VmInstruction::Mod(operand.0));
                            // unimplemented!("mod operator")
                        },
                    }
                },
                Instruction::Jump { label } => {
                    self.instruction_manager.translate_jump(label, VmInstruction::Jump);
                },
                Instruction::JNegative { label } => {
                    self.instruction_manager.translate_jump(label, VmInstruction::Jneg);
                },
                Instruction::JPositive { label } => {
                    self.instruction_manager.translate_jump(label, VmInstruction::Jpos);
                },
                Instruction::JZero { label } => {
                    self.instruction_manager.translate_jump(label, VmInstruction::Jzero);
                },
                Instruction::Get => self.instruction_manager.target_instructions.push(VmInstruction::Get),
                Instruction::Put => self.instruction_manager.target_instructions.push(VmInstruction::Put),
            }
        }

        self.instruction_manager.target_instructions.push(VmInstruction::Halt);

        println!("{:?}", self.instruction_manager.label_positions);

        self.instruction_manager.target_instructions
    }
}
