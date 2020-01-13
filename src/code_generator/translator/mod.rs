use crate::code_generator::intermediate::{Context, Instruction, Access, Variable, UniqueVariable, VariableIndex};
use ::virtual_machine::instruction::Instruction as VmInstruction;
use parser::ast::ExprOp;
use std::collections::BTreeMap;
use virtual_machine::interpreter::MemoryValue;
use std::cmp::Ordering;
use std::convert::TryInto;
use virtual_machine::interpreter;
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MemoryLocation(pub usize);

impl std::ops::Add<usize> for MemoryLocation {
    type Output = MemoryLocation;

    fn add(self, rhs: usize) -> Self::Output {
        MemoryLocation(self.0 + rhs)
    }
}

impl std::ops::AddAssign<usize> for MemoryLocation {
    fn add_assign(&mut self, rhs: usize) {
        self.0 += rhs
    }
}

type MemoryStorage = BTreeMap<VariableIndex, (MemoryLocation, Option<i64>)>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct MemoryRange(MemoryLocation, MemoryLocation);

impl MemoryRange {
    pub fn new(start: usize, end: usize) -> Self {
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
            let last = self.segments.arrays.as_ref().unwrap().1 + 1usize;
            self.segments.variables = Some(MemoryRange(last, last));
            last
        };

        self.storage.insert(index, (last, value));
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

fn allocate_memory(context: &Context) -> Memory {
    if context.variables().is_empty() {
        return Memory::new();
    }

    let mut variables: Vec<_> = context.variables().iter().map(|v| v).collect();
    variables.sort_unstable_by(|&a, &b| compare_variables(a, b));

    let middle = variables.binary_search_by(|&a| {
        match a.variable() {
            Variable::Array { .. } => Ordering::Less,
            Variable::Unit { .. } => Ordering::Greater,
        }
    }).expect_err("incorrect ordering function");

    println!("{}, {:?}", middle, variables);

    let mut memory = Memory::new();

    let (mut iter, arrays_segment_end) = if middle > 0 {
        let arrays_segment_end: usize = variables.iter().take(middle).map(|&arr| arr.variable().size()).sum();
        memory.segments.arrays = Some(MemoryRange(MemoryLocation(1), MemoryLocation(arrays_segment_end)));

        let mut iter = variables.iter();
        let arrays = iter.by_ref().take(middle);

        let array_base_indexes = arrays.scan(1, |first, &arr| {
            let start_index = *first;
            *first += arr.variable().size();
            if let Variable::Array { start, end, .. } = arr.variable() {
                Some((arr, start_index as i64 - *start))
            } else {
                panic!("incorrect variable order");
            }
        });

        for (ind, (arr, base_index)) in array_base_indexes.enumerate() {
            memory.add_variable(arr.id(), Some(base_index));
        }

        (iter, arrays_segment_end)
    } else {
        (variables.iter(), 1)
    };

    for (ind, &var) in iter.enumerate() {
        memory.add_variable(var.id(), None);
    }

    memory
}

//fn translate_load(&mut self, access: Access) {
//    match access {
//        Access::Constant(constant) => {
//            let index = self.context.register_constant(constant);
//            self.emit(Instruction::LoadDirect { variable: index });
//        },
//        Access::Variable(variable) => {
//            self.emit(Instruction::LoadDirect { variable });
//        },
//        Access::ArrayStatic(arr, constant) => {
//            let constant_index = self.context.register_constant(constant);
//            let register = self.context.find_variable_by_name("p0").expect("nonexistent register").id();
//            self.emit(Instruction::LoadDirect { variable: arr });
//            self.emit(Instruction::Operation {
//                op: ExprOp::Plus,
//                operand: constant_index,
//            });
//            self.emit(Instruction::LoadIndirect { variable: register });
//        },
//        Access::ArrayDynamic(arr, index) => {
//            let register = self.context.find_variable_by_name("p0").expect("nonexistent register").id();
//            self.emit(Instruction::LoadDirect { variable: arr });
//            self.emit(Instruction::Operation {
//                op: ExprOp::Plus,
//                operand: index,
//            });
//            self.emit(Instruction::LoadIndirect { variable: register });
//        },
//    }
//}

fn generate_constants(context: &Context, memory: &mut Memory) -> Vec<VmInstruction> {
    for (constant, index) in context.constants() {
        let value = memory.storage.get_mut(index).expect("constant not in memory");
        value.1 = Some(constant.value());
    }

    let mut to_generate: Vec<_> = memory.storage.iter().filter_map(|(_, (loc, val))| {
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

    println!("{:?}", to_generate);

    let mut generating_instructions = vec![];



    for (loc, val) in to_generate {

    }

    generating_instructions
}

pub fn translate(context: Context) -> Vec<VmInstruction> {
    let mut translated = Vec::with_capacity(context.instructions().len() * 3);
    let mut label_positions = BTreeMap::new();

    let mut locations = allocate_memory(&context);

    generate_constants(&context, &mut locations);

    for instruction in context.instructions() {
        match instruction {
            Instruction::Label { label } => {
                label_positions.insert(label, translated.len());
            },
            Instruction::Load { .. } => {},
            Instruction::PreStore { .. } => {},
            Instruction::Store { .. } => {},
            Instruction::Operation { op, operand } => {
                match op {
                    ExprOp::Plus => translated.push(VmInstruction::Add(operand.value() as u64)),
                    ExprOp::Minus => {},
                    ExprOp::Times => {},
                    ExprOp::Div => {},
                    ExprOp::Mod => {},
                }
            },
            Instruction::Jump { .. } => {},
            Instruction::JNegative { .. } => {},
            Instruction::JPositive { .. } => {},
            Instruction::JZero { .. } => {},
            Instruction::Get => translated.push(VmInstruction::Get),
            Instruction::Put => translated.push(VmInstruction::Put),
        }
    }

    translated
}
