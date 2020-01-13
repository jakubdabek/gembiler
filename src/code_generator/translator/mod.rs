use crate::code_generator::intermediate::{Context, Instruction, Access, Variable, UniqueVariable, VariableIndex};
use ::virtual_machine::instruction::Instruction as VmInstruction;
use parser::ast::ExprOp;
use std::collections::BTreeMap;
use virtual_machine::interpreter::MemoryValue;
use std::cmp::Ordering;
use std::convert::TryInto;

struct MemoryLocation(pub usize);

type Memory = BTreeMap<VariableIndex, (MemoryLocation, Option<MemoryValue>)>;

fn compare_variables(a: &UniqueVariable, b: &UniqueVariable) -> Ordering {
    match (a.variable(), b.variable()) {
        (v1@Variable::Array{..}, v2@Variable::Array{..}) => v1.size().cmp(&v2.size()),
        (Variable::Unit{ name: name1 }, Variable::Unit{ name: name2 }) => name1.cmp(name2),
        (Variable::Array{..}, Variable::Unit{..}) => Ordering::Greater,
        (Variable::Unit{..}, Variable::Array{..}) => Ordering::Less,
    }
}

fn allocate_memory(context: &Context) -> Memory {
    if context.variables().is_empty() {
        return Default::default();
    }

    let mut variables: Vec<_> = context.variables().iter().map(|v| v).collect();
    variables.sort_unstable_by(|&a, &b| compare_variables(a, b));

    let middle = variables.binary_search_by(|&a| {
        match a.variable() {
            Variable::Array { .. } => Ordering::Less,
            Variable::Unit { .. } => Ordering::Greater,
        }
    }).expect_err("incorrect ordering function");

    let mut locations = BTreeMap::new();

    let (mut iter, arrays_segment_end) = if middle > 0 {
        let arrays_segment_end: usize = variables.iter().take(middle - 1).map(|&arr| arr.variable().size()).sum();

        let mut iter = variables.iter();
        let arrays = iter.by_ref().take(middle - 1);

        let array_base_indexes = arrays.scan(1, |first, &arr| {
            let start_index = *first;
            *first += arr.variable().size();
            if let Variable::Array { start, end, .. } = arr.variable() {
                Some((arr, start_index - *start as usize))
            } else {
                panic!("incorrect variable order");
            }
        });

        for (ind, (arr, base_index)) in array_base_indexes.enumerate() {
            locations.insert(arr.id(), (MemoryLocation(arrays_segment_end + ind + 1), Some(base_index.try_into().unwrap())));
        }

        (iter, arrays_segment_end)
    } else {
        (variables.iter(), 1)
    };


    let vars_segment_start = locations.len() + arrays_segment_end;

    for (ind, &var) in iter.enumerate() {
        locations.insert(var.id(), (MemoryLocation(vars_segment_start + ind), None));
    }

    locations
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

fn generate_constants(context: &Context, locations: &mut Memory) {
    for (constant, index) in context.constants() {
        let value = locations.get_mut(index).expect("constant not in memory");
        value.1 = Some(constant.value());
    }
}

pub fn translate(context: Context) -> Vec<VmInstruction> {
    let mut translated = vec![];
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
