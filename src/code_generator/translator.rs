use crate::code_generator::intermediate::{Access, Constant, Context, Instruction, Label, UniqueVariable, Variable, VariableIndex, OperationType};
use ::virtual_machine::instruction::Instruction as VmInstruction;
use std::cmp::Ordering;
use std::collections::BTreeMap;

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

    fn add_variable(&mut self, index: VariableIndex, value: Option<i64>) -> MemoryLocation {
        let last = if let Some(MemoryRange(_, ref mut end)) = self.segments.variables {
            *end += 1;
            *end
        } else {
            let last = self
                .segments
                .arrays
                .as_ref()
                .map_or(MemoryLocation(0), |s| s.1)
                + 1;
            self.segments.variables = Some(MemoryRange(last, last));
            last
        };

        self.storage.insert(index, (last, value));

        last
    }

    fn get_location(&self, index: VariableIndex) -> MemoryLocation {
        self.storage[&index].0
    }
}

fn compare_variables(a: &UniqueVariable, b: &UniqueVariable) -> Ordering {
    match (a.variable(), b.variable()) {
        (v1 @ Variable::Array { .. }, v2 @ Variable::Array { .. }) => v1.size().cmp(&v2.size()),
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

#[allow(non_snake_case)]
impl InstructionManager {
    fn fix_label(&mut self, instruction_ptr: usize, target_pointer: u64) {
        match self.target_instructions[instruction_ptr] {
            VmInstruction::Jump(ref mut target)
            | VmInstruction::Jpos(ref mut target)
            | VmInstruction::Jneg(ref mut target)
            | VmInstruction::Jzero(ref mut target) => {
                *target = target_pointer;
            }
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

    fn instr_Get(&mut self) {
        self.target_instructions.push(VmInstruction::Get);
    }

    fn instr_Put(&mut self) {
        self.target_instructions.push(VmInstruction::Put);
    }

    fn instr_Load(&mut self, operand: MemoryLocation) {
        self.target_instructions
            .push(VmInstruction::Load(operand.0));
    }

    fn instr_Loadi(&mut self, operand: MemoryLocation) {
        self.target_instructions
            .push(VmInstruction::Loadi(operand.0));
    }

    fn instr_Store(&mut self, operand: MemoryLocation) {
        self.target_instructions
            .push(VmInstruction::Store(operand.0));
    }

    fn instr_Storei(&mut self, operand: MemoryLocation) {
        self.target_instructions
            .push(VmInstruction::Storei(operand.0));
    }

    fn instr_Add(&mut self, operand: MemoryLocation) {
        self.target_instructions.push(VmInstruction::Add(operand.0));
    }

    fn instr_Sub(&mut self, operand: MemoryLocation) {
        self.target_instructions.push(VmInstruction::Sub(operand.0));
    }

    fn instr_Shift(&mut self, operand: MemoryLocation) {
        self.target_instructions
            .push(VmInstruction::Shift(operand.0));
    }

    fn instr_Inc(&mut self) {
        self.target_instructions.push(VmInstruction::Inc);
    }

    fn instr_Dec(&mut self) {
        self.target_instructions.push(VmInstruction::Dec);
    }

    fn instr_Halt(&mut self) {
        self.target_instructions.push(VmInstruction::Halt);
    }
}

pub struct Generator {
    context: Context,
    memory: Memory,
    instruction_manager: InstructionManager,
}

#[allow(dead_code)]
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

        let middle = variables
            .binary_search_by(|&a| match a.variable() {
                Variable::Array { .. } => Ordering::Less,
                Variable::Unit { .. } => Ordering::Greater,
            })
            .expect_err("incorrect ordering function");

        let iter = if middle > 0 {
            let arrays_segment_end: usize = variables
                .iter()
                .take(middle)
                .map(|&arr| arr.variable().size())
                .sum();
            self.memory.segments.arrays = Some(MemoryRange::new(1, arrays_segment_end as u64));

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

            for (arr, base_index) in array_base_indexes {
                self.memory.add_variable(arr.id(), Some(base_index));
            }

            iter
        } else {
            variables.iter()
        };

        for &var in iter {
            self.memory.add_variable(var.id(), None);
        }
    }

    fn get_constant_location(&self, value: i64) -> MemoryLocation {
        let ind = self.context.get_constant_index(&Constant(value));
        self.memory
            .storage
            .get(&ind)
            .expect(format!("constant {} has not been generated", value).as_str())
            .0
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
                self.instruction_manager
                    .target_instructions
                    .push(grow_instr);
            }
            self.instruction_manager.instr_Store(location);
            for _ in 0..value.abs() {
                self.instruction_manager
                    .target_instructions
                    .push(shrink_instr);
            }
        } else {
            let leading_zeros = abs.leading_zeros();
            let mut abs = abs.reverse_bits();

            let one_const = self.get_constant_location(1);

            let grow_instr = if value.is_positive() {
                VmInstruction::Inc
            } else {
                VmInstruction::Dec
            };

            while abs & 1 == 0 {
                abs >>= 1;
            }

            for _ in 0..(64 - leading_zeros - 1) {
                if abs & 1 == 1 {
                    self.instruction_manager
                        .target_instructions
                        .push(grow_instr);
                }

                self.instruction_manager.instr_Shift(one_const);
                abs >>= 1;
            }
            if abs & 1 == 1 {
                self.instruction_manager
                    .target_instructions
                    .push(grow_instr);
            }

            self.instruction_manager.instr_Store(location);
            self.instruction_manager.instr_Sub(MemoryLocation(0));
        }
    }

    fn generate_constants(&mut self) {
        for (constant, index) in self.context.constants() {
            let value = self
                .memory
                .storage
                .get_mut(index)
                .expect("constant not in memory");
            value.1 = Some(constant.value());
        }

        let mut to_generate: Vec<_> = self
            .memory
            .storage
            .iter()
            .filter_map(|(_, &(loc, val))| val.map(|val| (loc, val)))
            .collect();

        to_generate.sort_unstable_by(|(_, val1), (_, val2)| {
            let cmp = val1.abs().cmp(&val2.abs());
            if cmp == Ordering::Equal {
                val1.cmp(val2)
            } else {
                cmp
            }
        });

        if cfg!(debug_assertions) {
            println!("generating constants: {:?}", to_generate);
        }

        self.instruction_manager.instr_Sub(MemoryLocation(0));

        for (loc, val) in to_generate {
            self.generate_constant(val, loc);
        }
    }

    fn register_temp(&mut self, name: &str) -> VariableIndex {
        self.context.add_variable(Variable::Unit {
            name: String::from("tmp$") + name,
        })
    }

    fn get_or_register_temp(&mut self, name: &str) -> MemoryLocation {
        let location = self
            .context
            .find_variable_by_name((String::from("tmp$") + name).as_str())
            .map(|v| self.memory.get_location(v.id()))
            .unwrap_or_else(|| {
                let new_ind = self.register_temp(name);
                self.memory.add_variable(new_ind, None)
            });

        location
    }

    fn translate_load_zero(&mut self) {
        self.instruction_manager.instr_Sub(MemoryLocation(0));
    }

    fn translate_optimized_multiplication(&mut self, left: &Access, right: &Access) -> bool {
        match (left, right) {
            (Access::Constant(c), other)
            | (other, Access::Constant(c)) => {
                match c.value() {
                    0 => {
                        self.instruction_manager.instr_Sub(MemoryLocation(0));
                        true
                    },
                    1 => {
                        self.translate_load_access(other);
                        true
                    },
                    -1 => {
                        self.translate_load_access(other);
                        self.translate_neg_tmp();
                        true
                    }
                    2 => {
                        self.translate_load_access(other);
                        self.instruction_manager.instr_Shift(self.get_constant_location(1));
                        true
                    }
                    -2 => {
                        self.translate_load_access(other);
                        self.instruction_manager.instr_Shift(self.get_constant_location(1));
                        self.translate_neg_tmp();
                        true
                    }
                    // n if (n.abs() as u64).is_power_of_two() => {}
                    _ => false
                }
            }
            _ => false,
        }
    }

    fn translate_optimized_div_mod(&mut self, left: &Access, right: &Access, div: bool) -> bool {
        match (left, right) {
            (Access::Constant(c1), Access::Constant(c2)) => {
                if div {
                    let v1 = c1.value();
                    let v2 = c2.value();

                    match (v1.signum(), v2.signum()) {
                        (0, _) | (_, 0) => {
                            self.translate_load_zero();
                            true
                        },
                        _ => false
                    }
                } else {
                    false
                }
            }
            (Access::Constant(c), _other) => {
                match c.value() {
                    0 => {
                        self.instruction_manager.instr_Sub(MemoryLocation(0));
                        true
                    }
                    // n if (n.abs() as u64).is_power_of_two() => {}
                    _ => false
                }
            }
            (other, Access::Constant(c)) => {
                match c.value() {
                    0 => {
                        self.translate_load_zero();
                        true
                    },
                    1 => {
                        self.translate_load_access(other);
                        true
                    },
                    -1 => {
                        self.translate_load_access(other);
                        self.translate_neg_tmp();
                        true
                    }
                    2 => {
                        if div {
                            self.translate_load_access(other);
                            self.instruction_manager.instr_Shift(self.get_constant_location(-1));
                            true
                        } else {
                            false
                        }
                    }
                    -2 => {
                        if div {
                            self.translate_load_access(other);
                            self.instruction_manager.instr_Shift(self.get_constant_location(-1));
                            self.translate_neg_tmp();
                            true
                        } else {
                            false
                        }
                    }
                    // n if (n.abs() as u64).is_power_of_two() => {}
                    _ => false
                }
            }
            _ => false,
        }
    }


    fn translate_multiplication(&mut self, left: &Access, right: &Access) {
        /*
            if b == 0 { goto end }
            if b < 0 {
              b = -b
              a = -a
            }
            result = 0
            while b > 0 {
              if lsb(b) == 1 {
                result += a
              }
              b >>= 1
              a <<= 1
            }
            p0 = result
            end: return p0
        */

        if self.translate_optimized_multiplication(left, right) {
            return;
        }

        let left_tmp = self.get_or_register_temp("mul_left");
        let right_tmp = self.get_or_register_temp("mul_right");
        let tmp = self.get_or_register_temp("1");
        let result = self.get_or_register_temp("mul_result");
        let const_1 = self.get_constant_location(1);
        let const_neg_1 = self.get_constant_location(-1);

        let label_start = self.context.new_label();
        let label_main = self.context.new_label();
        let label_step = self.context.new_label();
        let label_end = self.context.new_label();
        let label_real_end = self.context.new_label();

        self.translate_load_access(left);
        self.instruction_manager.instr_Store(left_tmp);
        self.translate_load_access(right);
        self.instruction_manager.instr_Store(right_tmp);
        self.instruction_manager
            .translate_jump(&label_real_end, VmInstruction::Jzero);
        self.instruction_manager
            .translate_jump(&label_start, VmInstruction::Jpos);

        self.translate_neg_tmp();
        self.instruction_manager.instr_Store(right_tmp);
        self.instruction_manager.instr_Load(left_tmp);
        self.translate_neg(left_tmp);
        self.instruction_manager.instr_Store(left_tmp);

        self.instruction_manager.instr_Load(right_tmp);

        self.instruction_manager.translate_label(&label_start);
        self.instruction_manager.instr_Sub(MemoryLocation(0));
        self.instruction_manager.instr_Store(result);

        self.instruction_manager.translate_label(&label_main);
        self.instruction_manager.instr_Load(right_tmp);
        self.instruction_manager.instr_Store(tmp);
        self.instruction_manager.instr_Shift(const_neg_1);
        self.instruction_manager.instr_Shift(const_1);
        self.instruction_manager.instr_Sub(tmp);
        self.instruction_manager
            .translate_jump(&label_step, VmInstruction::Jzero);

        self.instruction_manager.instr_Load(left_tmp);
        self.instruction_manager.instr_Add(result);
        self.instruction_manager.instr_Store(result);

        self.instruction_manager.translate_label(&label_step);
        self.instruction_manager.instr_Load(right_tmp);
        self.instruction_manager.instr_Shift(const_neg_1);
        self.instruction_manager
            .translate_jump(&label_end, VmInstruction::Jzero);

        self.instruction_manager.instr_Store(right_tmp);
        self.instruction_manager.instr_Load(left_tmp);
        self.instruction_manager.instr_Shift(const_1);
        self.instruction_manager.instr_Store(left_tmp);
        self.instruction_manager
            .translate_jump(&label_main, VmInstruction::Jump);

        self.instruction_manager.translate_label(&label_end);
        self.instruction_manager.instr_Load(result);

        self.instruction_manager.translate_label(&label_real_end);
    }

    fn translate_log(&mut self) {
        let num = self.get_or_register_temp("num");
        let value = self.get_or_register_temp("value");
        let const_neg_1 = self.get_constant_location(-1);

        let label_start = self.context.new_label();
        let label_end = self.context.new_label();

        self.instruction_manager.instr_Store(num);
        self.instruction_manager.instr_Sub(MemoryLocation(0));
        self.instruction_manager.instr_Store(value);

        self.instruction_manager.translate_label(&label_start);
        self.instruction_manager.instr_Load(num);
        self.instruction_manager.instr_Shift(const_neg_1);
        self.instruction_manager.instr_Store(num);
        self.instruction_manager
            .translate_jump(&label_end, VmInstruction::Jzero);

        self.instruction_manager.instr_Load(value);
        self.instruction_manager.instr_Inc();
        self.instruction_manager.instr_Store(value);
        self.instruction_manager
            .translate_jump(&label_start, VmInstruction::Jump);

        self.instruction_manager.translate_label(&label_end);
        self.instruction_manager.instr_Load(value);
    }

    fn translate_abs(&mut self, original: MemoryLocation) {
        let label_pos = self.context.new_label();

        self.instruction_manager
            .translate_jump(&label_pos, VmInstruction::Jpos);
        self.translate_neg(original);
        self.instruction_manager.translate_label(&label_pos);
    }

    fn translate_abs_tmp(&mut self) {
        let tmp = self.get_or_register_temp("abs");
        self.instruction_manager.instr_Store(tmp);
        self.translate_abs(tmp);
    }

    fn translate_neg(&mut self, original: MemoryLocation) {
        self.instruction_manager.instr_Sub(original);
        self.instruction_manager.instr_Sub(original);
    }

    fn translate_neg_tmp(&mut self) {
        let tmp = self.get_or_register_temp("neg");
        self.instruction_manager.instr_Store(tmp);
        self.translate_neg(tmp);
    }

    fn translate_div_mod(&mut self, left: &Access, right: &Access, div: bool) {
        /*
            if(divisor == 0)
                return (0, 0);

            remain         = dividend; // The left-hand side of division, i.e. what is being divided
            scaled_divisor = divisor;  // The right-hand side of division
            result   = 0;
            multiple = 1;

            while(scaled_divisor < dividend)
            {
                scaled_divisor = scaled_divisor + scaled_divisor; // Multiply by two.
                multiple       = multiple       + multiple;       // Multiply by two.
                // You can also use binary shift-by-left here (i.e. multiple = multiple << 1).
            }

            do {
                if(remain >= scaled_divisor)
                {
                    remain = remain - scaled_divisor;
                    result = result + multiple;
                }
                scaled_divisor = scaled_divisor >> 1; // Divide by two.
                multiple       = multiple       >> 1;
            } while(multiple != 0);

            return (result, remain)
        */

        if self.translate_optimized_div_mod(left, right, div) {
            return;
        }

        let label_while_condition = self.context.new_label();
        let label_while_body = self.context.new_label();
        // let label_after_while = self.context.new_label();
        let label_after_if = self.context.new_label();
        let label_do_body = self.context.new_label();
        let label_after_do = self.context.new_label();
        // let label_divisor_1 = self.context.new_label();
        // let label_divisor_2 = self.context.new_label();
        // let label_divisor_neg_1 = self.context.new_label();
        // let label_divisor_neg_2 = self.context.new_label();
        let label_end = self.context.new_label();

        let const_1 = self.get_constant_location(1);
        let const_neg_1 = self.get_constant_location(-1);

        let original_dividend = self.get_or_register_temp("original_dividend");
        let original_divisor = self.get_or_register_temp("original_divisor");
        let dividend_abs = self.get_or_register_temp("dividend_abs");
        let scaled_divisor = self.get_or_register_temp("scaled_divisor");
        let remain = self.get_or_register_temp("remain");
        let result = self.get_or_register_temp("div_result");
        let multiple = self.get_or_register_temp("div_multiple");

        self.translate_load_access(left);
        self.instruction_manager.instr_Store(original_dividend);
        self.translate_abs(original_dividend);
        self.instruction_manager.instr_Store(dividend_abs);
        self.instruction_manager.instr_Store(remain);

        self.translate_load_access(right);
        self.instruction_manager
            .translate_jump(&label_end, VmInstruction::Jzero);
        // self.instruction_manager.instr_Dec();
        // self.instruction_manager.translate_jump(&label_divisor_1, VmInstruction::Jzero);
        // self.instruction_manager.instr_Dec();
        // self.instruction_manager.translate_jump(&label_divisor_2, VmInstruction::Jzero);
        // self.instruction_manager.instr_Inc();
        // self.instruction_manager.instr_Inc();
        // self.instruction_manager.instr_Inc();
        // self.instruction_manager.translate_jump(&label_divisor_neg_1, VmInstruction::Jzero);
        // self.instruction_manager.instr_Inc();
        // self.instruction_manager.translate_jump(&label_divisor_neg_2, VmInstruction::Jzero);
        // self.instruction_manager.instr_Dec();
        // self.instruction_manager.instr_Dec();

        self.instruction_manager.instr_Store(original_divisor);
        self.translate_abs(original_divisor);
        self.instruction_manager.instr_Store(scaled_divisor);
        self.instruction_manager.instr_Sub(MemoryLocation(0));
        self.instruction_manager.instr_Store(result);
        self.instruction_manager.instr_Inc();
        self.instruction_manager.instr_Store(multiple);

        self.instruction_manager.instr_Load(scaled_divisor);
        self.instruction_manager
            .translate_jump(&label_while_condition, VmInstruction::Jump);

        self.instruction_manager.translate_label(&label_while_body);
        self.instruction_manager.instr_Load(multiple);
        self.instruction_manager.instr_Shift(const_1);
        self.instruction_manager.instr_Store(multiple);
        self.instruction_manager.instr_Load(scaled_divisor);
        self.instruction_manager.instr_Shift(const_1);
        self.instruction_manager.instr_Store(scaled_divisor);

        self.instruction_manager
            .translate_label(&label_while_condition);
        // assert(p0 == scaled_divisor)
        self.instruction_manager.instr_Sub(dividend_abs);
        self.instruction_manager
            .translate_jump(&label_while_body, VmInstruction::Jneg);
        // self.instruction_manager.translate_jump(&label_after_while, VmInstruction::Jump);
        // self.instruction_manager.translate_label(&label_after_while);

        self.instruction_manager.translate_label(&label_do_body);
        self.instruction_manager.instr_Load(remain);
        self.instruction_manager.instr_Sub(scaled_divisor);
        self.instruction_manager
            .translate_jump(&label_after_if, VmInstruction::Jneg);

        // assert(p0 == remain - scaled_divisor)
        self.instruction_manager.instr_Store(remain);
        self.instruction_manager.instr_Load(result);
        self.instruction_manager.instr_Add(multiple);
        self.instruction_manager.instr_Store(result);

        self.instruction_manager.translate_label(&label_after_if);
        self.instruction_manager.instr_Load(multiple);
        self.instruction_manager.instr_Shift(const_neg_1);
        self.instruction_manager
            .translate_jump(&label_after_do, VmInstruction::Jzero);
        self.instruction_manager.instr_Store(multiple);
        self.instruction_manager.instr_Load(scaled_divisor);
        self.instruction_manager.instr_Shift(const_neg_1);
        self.instruction_manager.instr_Store(scaled_divisor);
        self.instruction_manager
            .translate_jump(&label_do_body, VmInstruction::Jump);

        self.instruction_manager.translate_label(&label_after_do);

        if div {
            let label_remain_zero = self.context.new_label();
            let label_dividend_neg = self.context.new_label();
            let label_only_divisor_neg = self.context.new_label();
            let label_both_neg = self.context.new_label();

            self.instruction_manager.instr_Load(original_dividend);
            self.instruction_manager
                .translate_jump(&label_dividend_neg, VmInstruction::Jneg);
            // fallthrough

            // (+ / ?)
            self.instruction_manager.instr_Load(original_divisor);
            self.instruction_manager
                .translate_jump(&label_only_divisor_neg, VmInstruction::Jneg);
            // fallthrough

            // (+ / +) or (- / -)
            self.instruction_manager.translate_label(&label_both_neg);
            self.instruction_manager.instr_Load(result);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);

            // (- / ?)
            self.instruction_manager
                .translate_label(&label_dividend_neg);
            self.instruction_manager.instr_Load(original_divisor);
            self.instruction_manager
                .translate_jump(&label_both_neg, VmInstruction::Jneg);
            // fallthrough

            // (+ / -) or (- / +)
            self.instruction_manager
                .translate_label(&label_only_divisor_neg);
            self.instruction_manager.instr_Load(remain);
            self.instruction_manager
                .translate_jump(&label_remain_zero, VmInstruction::Jzero);
            self.instruction_manager.instr_Load(result);
            self.translate_neg(result);
            self.instruction_manager.instr_Dec();
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);

            self.instruction_manager.translate_label(&label_remain_zero);
            self.instruction_manager.instr_Load(result);
            self.translate_neg(result);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);
        } else {
            let label_dividend_neg = self.context.new_label();
            let label_only_divisor_neg = self.context.new_label();
            let label_both_neg = self.context.new_label();

            self.instruction_manager.instr_Load(remain);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jzero);

            self.instruction_manager.instr_Load(original_dividend);
            self.instruction_manager
                .translate_jump(&label_dividend_neg, VmInstruction::Jneg);
            // fallthrough

            // (+ % ?)
            self.instruction_manager.instr_Load(original_divisor);
            self.instruction_manager
                .translate_jump(&label_only_divisor_neg, VmInstruction::Jneg);
            // fallthrough

            // (+ % +)
            self.instruction_manager.instr_Load(remain);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);

            // (- % ?)
            self.instruction_manager
                .translate_label(&label_dividend_neg);
            self.instruction_manager.instr_Load(original_divisor);
            self.instruction_manager
                .translate_jump(&label_both_neg, VmInstruction::Jneg);
            // fallthrough

            // (- % +)
            self.instruction_manager.instr_Load(remain);
            self.instruction_manager.instr_Sub(original_divisor);
            self.translate_neg_tmp();
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);

            // (+ % -)
            self.instruction_manager
                .translate_label(&label_only_divisor_neg);
            self.instruction_manager.instr_Load(remain);
            self.instruction_manager.instr_Add(original_divisor);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);

            // (- % -)
            self.instruction_manager.translate_label(&label_both_neg);
            self.instruction_manager.instr_Load(remain);
            self.translate_neg(remain);
            self.instruction_manager
                .translate_jump(&label_end, VmInstruction::Jump);
        }

        self.instruction_manager.translate_label(&label_end);
    }

    fn translate_load_access(&mut self, access: &Access) {
        match access {
            Access::Constant(c) => {
                let loc = self.get_constant_location(c.value());
                self.instruction_manager.instr_Load(loc);
            }
            Access::Variable(ind) => {
                let loc = self.memory.get_location(*ind);
                self.instruction_manager.instr_Load(loc);
            }
            Access::ArrayStatic(arr, c) => {
                let real_arr_loc = self.memory.storage[arr].1.expect("unallocated array");
                self.instruction_manager
                    .instr_Load(MemoryLocation((real_arr_loc + c.value()) as u64));
            }
            Access::ArrayDynamic(arr, ind) => {
                let arr_loc = self.memory.get_location(*arr);
                let ind_loc = self.memory.get_location(*ind);

                self.instruction_manager.instr_Load(arr_loc);
                self.instruction_manager.instr_Add(ind_loc);
                self.instruction_manager.instr_Loadi(MemoryLocation(0));
            }
        }
    }

    fn translate_store_access(&mut self, access: &Access) {
        match access {
            Access::Constant(_) => panic!("can't store into a constant"),
            Access::Variable(ind) => {
                let loc = self.memory.get_location(*ind);
                self.instruction_manager.instr_Store(loc);
            }
            Access::ArrayStatic(arr, c) => {
                let real_arr_loc = self.memory.storage[arr].1.expect("unallocated array");
                self.instruction_manager
                    .instr_Store(MemoryLocation((real_arr_loc + c.value()) as u64));
            }
            Access::ArrayDynamic(arr, ind) => {
                let tmp1 = self.get_or_register_temp("store_tmp1");
                self.instruction_manager.instr_Store(tmp1);

                let arr_loc = self.memory.get_location(*arr);
                let ind_loc = self.memory.get_location(*ind);

                self.instruction_manager.instr_Load(arr_loc);
                self.instruction_manager.instr_Add(ind_loc);

                let tmp2 = self.get_or_register_temp("store_tmp2");
                self.instruction_manager.instr_Store(tmp2);

                self.instruction_manager.instr_Load(tmp1);
                self.instruction_manager.instr_Storei(tmp2);
            }
        }
    }

    fn translate_plus(&mut self, left: &Access, right: &Access) {
        let optimized = match (left, right) {
            (Access::Constant(c), other)
            | (other, Access::Constant(c)) => {
                match c.value() {
                    0 => {
                        self.translate_load_access(other);
                        true
                    },
                    n if n > 0 && n <= 10 => {
                        self.translate_load_access(other);
                        for _ in 0..n {
                            self.instruction_manager.instr_Inc();
                        }
                        true
                    }
                    n if n < 0 && n >= -10 => {
                        self.translate_load_access(other);
                        for _ in 0..n.abs() {
                            self.instruction_manager.instr_Dec();
                        }
                        true
                    }
                    _ => false
                }
            }
            _ => false,
        };

        if !optimized {
            self.translate_simple_bin_op(left, right, InstructionManager::instr_Add);
        }
    }

    fn translate_minus(&mut self, left: &Access, right: &Access) {
        let optimized = match (left, right) {
            (other, Access::Constant(c)) => {
                match c.value() {
                    0 => {
                        self.translate_load_access(other);
                        true
                    },
                    n if n > 0 && n <= 10 => {
                        self.translate_load_access(other);
                        for _ in 0..n {
                            self.instruction_manager.instr_Dec();
                        }
                        true
                    }
                    n if n < 0 && n >= -10 => {
                        self.translate_load_access(other);
                        for _ in 0..n.abs() {
                            self.instruction_manager.instr_Inc();
                        }
                        true
                    }
                    _ => false
                }
            }
            (Access::Constant(c), other) => {
                match c.value() {
                    0 => {
                        self.translate_load_access(other);
                        self.translate_neg_tmp();
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        };

        if !optimized {
            self.translate_simple_bin_op(left, right, InstructionManager::instr_Sub);
        }
    }

    fn translate_simple_bin_op(&mut self, left: &Access, right: &Access, op: fn(&mut InstructionManager, MemoryLocation)) {
        self.translate_load_access(right);
        let tmp = self.get_or_register_temp("bin_op");
        self.instruction_manager.instr_Store(tmp);

        self.translate_load_access(left);
        op(&mut self.instruction_manager, tmp);
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

        self.allocate_memory();
        self.generate_constants();

        let ir_instructions = self.context.instructions().to_vec();

        for instruction in &ir_instructions {
            match instruction {
                Instruction::Label { label } => {
                    self.instruction_manager.translate_label(label);
                }
                Instruction::Load { access } => self.translate_load_access(access),
                Instruction::PreStore { access } => {
                    match access {
                        Access::Constant(_) | Access::Variable(_) | Access::ArrayStatic(_, _) => (),
                        Access::ArrayDynamic(_, _) => (), // unimplemented!(),
                    }
                }
                Instruction::Store { access } => self.translate_store_access(access),
                Instruction::Operation { left, op, right } => {
                    match op {
                        OperationType::Plus => {
                            self.translate_plus(left, right)
                        }
                        OperationType::Minus => {
                           self.translate_minus(left, right)
                        }
                        OperationType::Shift => {
                            self.translate_simple_bin_op(left, right, InstructionManager::instr_Shift)
                        }
                        OperationType::Times => {
                            self.translate_multiplication(left, right);
                        }
                        OperationType::Div => {
                            self.translate_div_mod(left, right, true);
                        }
                        OperationType::Mod => {
                            self.translate_div_mod(left, right, false);
                        }
                    }
                }
                Instruction::Jump { label } => {
                    self.instruction_manager
                        .translate_jump(label, VmInstruction::Jump);
                }
                Instruction::JNegative { label } => {
                    self.instruction_manager
                        .translate_jump(label, VmInstruction::Jneg);
                }
                Instruction::JPositive { label } => {
                    self.instruction_manager
                        .translate_jump(label, VmInstruction::Jpos);
                }
                Instruction::JZero { label } => {
                    self.instruction_manager
                        .translate_jump(label, VmInstruction::Jzero);
                }
                Instruction::Get => self.instruction_manager.instr_Get(),
                Instruction::Put => self.instruction_manager.instr_Put(),
            }
        }

        self.instruction_manager.instr_Halt();

        if cfg!(debug_assertions) {
            let vars = self
                .context
                .variables()
                .iter()
                .map(|v| (v, self.memory.get_location(v.id())));
            for x in vars {
                println!("{:?}", x);
            }
            println!("{:?}", self.instruction_manager.label_positions);
        }

        self.instruction_manager.target_instructions
    }
}
