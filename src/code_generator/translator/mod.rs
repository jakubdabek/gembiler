use crate::code_generator::intermediate::{Context, Instruction, Access, Variable, UniqueVariable, VariableIndex, Constant};
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

pub struct Generator {
    context: Context,
    memory: Memory,
    target_instructions: Vec<VmInstruction>,
}

impl Generator {
    pub fn new(context: Context) -> Self {
        let cap = context.instructions().len() * 4;
        Generator {
            context,
            memory: Memory::new(),
            target_instructions: Vec::with_capacity(cap),
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

    fn generate_constant(&mut self, value: i64, location: MemoryLocation) {
        let abs = value.abs();
        if abs < 10 {
            let (grow_instr, shrink_instr) = if value.is_positive() {
                (VmInstruction::Inc, VmInstruction::Dec)
            } else {
                (VmInstruction::Dec, VmInstruction::Inc)
            };

            for _ in 0..value.abs() {
                self.target_instructions.push(grow_instr);
            }
            self.target_instructions.push(VmInstruction::Store(location.0 as u64));
            for _ in 0..value.abs() {
                self.target_instructions.push(shrink_instr);
            }
        } else {
            let mut abs = abs.reverse_bits();

            let (shift_const, grow_instr) = if value.is_positive() {
                let ind = self.context.get_constant_index(&Constant(1));
                let c1 = self.memory.storage.get(&ind).expect("1 constant has not been generated").0;
                (c1, VmInstruction::Inc)
            } else {
                let ind = self.context.get_constant_index(&Constant(-1));
                let c_1 = self.memory.storage.get(&ind).expect("-1 constant has not been generated").0;
                (c_1, VmInstruction::Dec)
            };

            let shift_loc = shift_const.0;

            while abs & 1 == 0 {
                abs >>= 1;
            }

            while abs > 1 {
                if abs & 1 == 1 {
                    self.target_instructions.push(grow_instr);
                }

                self.target_instructions.push(VmInstruction::Shift(shift_loc));
            }

            if abs & 1 == 1 {
                self.target_instructions.push(grow_instr);
            }

            self.target_instructions.push(VmInstruction::Sub(0));
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

        self.target_instructions.push(VmInstruction::Sub(0));

        for (loc, val) in to_generate {
            self.generate_constant(val, loc);
        }
    }

    pub fn translate(mut self) -> Vec<VmInstruction> {
        let mut label_positions = BTreeMap::new();

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

        self.allocate_memory();
        self.generate_constants();

        println!("{:?}", self.context.variables());
        println!("{:?}", self.memory.storage);

        let mut backpatching_list: BTreeMap<_, Vec<_>> = BTreeMap::new();

        for instruction in self.context.instructions() {
            match instruction {
                Instruction::Label { label } => {
                    let pos = self.target_instructions.len() as u64;
                    label_positions.insert(label, pos);
                    if let Some(backlist) = backpatching_list.remove(&label) {
                        for i in backlist {
                            match self.target_instructions[i] {
                                VmInstruction::Jump(ref mut target)
                                | VmInstruction::Jpos(ref mut target)
                                | VmInstruction::Jneg(ref mut target)
                                | VmInstruction::Jzero(ref mut target) => {
                                    *target = pos;
                                },
                                _ => unreachable!(),
                            }
                        }
                    }
                },
                Instruction::Load { access } => {
                    match access {
                        Access::Constant(c) => {
                            let ind = self.context.get_constant_index(c);
                            let loc = self.memory.get_location(ind);
                            self.target_instructions.push(VmInstruction::Load(loc.0));
                        },
                        Access::Variable(ind) => {
                            let loc = self.memory.get_location(*ind);
                            self.target_instructions.push(VmInstruction::Load(loc.0));
                        },
                        Access::ArrayStatic(arr, c) => {
                            let real_arr_loc = self.memory.storage[arr].1.expect("unallocated array");
                            self.target_instructions.push(VmInstruction::Load((real_arr_loc + c.value()) as u64));
                        },
                        Access::ArrayDynamic(arr, ind) => {
                            let arr_loc = self.memory.get_location(*arr);
                            let ind_loc = self.memory.get_location(*ind);

                            self.target_instructions.push(VmInstruction::Load(arr_loc.0));
                            self.target_instructions.push(VmInstruction::Add(ind_loc.0));
                            self.target_instructions.push(VmInstruction::Loadi(0));
                        },
                    }
                },
                Instruction::PreStore { access } => {
                    match access {
                        Access::Constant(_)
                        | Access::Variable(_)
                        | Access::ArrayStatic(_, _) => (),
                        Access::ArrayDynamic(_, _) => unimplemented!(),
                    }
                },
                Instruction::Store { access } => {
                    match access {
                        Access::Constant(_) => panic!("can't store into a constant"),
                        Access::Variable(ind) => {
                            let loc = self.memory.get_location(*ind);
                            self.target_instructions.push(VmInstruction::Store(loc.0));
                        },
                        Access::ArrayStatic(arr, c) => {
                            let loc = self.memory.storage[arr].1.expect("unallocated array");
                            self.target_instructions.push(VmInstruction::Store((loc + c.value()) as u64));
                        },
                        Access::ArrayDynamic(arr, ind) => {
                            let tmp = self.context.find_variable_by_name("tmp$1").expect("tmp$1 unavailable");
                            let tmp_loc = self.memory.get_location(tmp.id());
                            self.target_instructions.push(VmInstruction::Store(tmp_loc.0));

                            let arr_loc = self.memory.get_location(*arr);
                            let ind_loc = self.memory.get_location(*ind);

                            self.target_instructions.push(VmInstruction::Load(arr_loc.0));
                            self.target_instructions.push(VmInstruction::Add(ind_loc.0));

                            let tmp2 = self.context.find_variable_by_name("tmp$2").expect("tmp$2 unavailable");
                            let tmp2_loc = self.memory.get_location(tmp2.id());
                            self.target_instructions.push(VmInstruction::Store(tmp2_loc.0));

                            self.target_instructions.push(VmInstruction::Load(tmp_loc.0));
                            self.target_instructions.push(VmInstruction::Storei(tmp2_loc.0));
                        },
                    }
                },
                Instruction::Operation { op, operand } => {
                    let loc = self.memory.get_location(*operand);
                    match op {
                        ExprOp::Plus => self.target_instructions.push(VmInstruction::Add(loc.0)),
                        ExprOp::Minus => self.target_instructions.push(VmInstruction::Sub(loc.0)),
                        ExprOp::Times => {
                            self.target_instructions.push(VmInstruction::Mul(loc.0));
                            // unimplemented!("times operator")
                        },
                        ExprOp::Div => {
                            self.target_instructions.push(VmInstruction::Div(loc.0));
                            // unimplemented!("div operator")
                        },
                        ExprOp::Mod => {
                            self.target_instructions.push(VmInstruction::Mod(loc.0));
                            // unimplemented!("mod operator")
                        },
                    }
                },
                Instruction::Jump { label } => {
                    if let Some(pos) = label_positions.get(label) {
                        self.target_instructions.push(VmInstruction::Jump(*pos));
                    } else {
                        let pos = self.target_instructions.len();
                        backpatching_list.entry(label).or_default().push(pos);
                        self.target_instructions.push(VmInstruction::Jump(u64::max_value()));
                    }
                },
                Instruction::JNegative { label } => {
                    if let Some(pos) = label_positions.get(label) {
                        self.target_instructions.push(VmInstruction::Jneg(*pos));
                    } else {
                        let pos = self.target_instructions.len();
                        backpatching_list.entry(label).or_default().push(pos);
                        self.target_instructions.push(VmInstruction::Jneg(u64::max_value()));
                    }
                },
                Instruction::JPositive { label } => {
                    if let Some(pos) = label_positions.get(label) {
                        self.target_instructions.push(VmInstruction::Jpos(*pos));
                    } else {
                        let pos = self.target_instructions.len();
                        backpatching_list.entry(label).or_default().push(pos);
                        self.target_instructions.push(VmInstruction::Jpos(u64::max_value()));
                    }
                },
                Instruction::JZero { label } => {
                    if let Some(pos) = label_positions.get(label) {
                        self.target_instructions.push(VmInstruction::Jzero(*pos));
                    } else {
                        let pos = self.target_instructions.len();
                        backpatching_list.entry(label).or_default().push(pos);
                        self.target_instructions.push(VmInstruction::Jzero(u64::max_value()));
                    }
                },
                Instruction::Get => self.target_instructions.push(VmInstruction::Get),
                Instruction::Put => self.target_instructions.push(VmInstruction::Put),
            }
        }

        self.target_instructions.push(VmInstruction::Halt);

        self.target_instructions
    }
}
