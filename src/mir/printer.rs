/*!
 * MIR Printer - Debug output and visualization
 * 
 * Implements pretty-printing for MIR modules and functions
 */

use super::{MirModule, MirFunction, BasicBlock, MirInstruction};
use std::fmt::Write;

/// MIR printer for debug output and visualization
pub struct MirPrinter {
    /// Indentation level
    #[allow(dead_code)]
    indent_level: usize,
    
    /// Whether to show detailed information
    verbose: bool,
    
    /// Whether to show line numbers
    show_line_numbers: bool,
}

impl MirPrinter {
    /// Create a new MIR printer with default settings
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            verbose: false,
            show_line_numbers: true,
        }
    }
    
    /// Create a verbose MIR printer
    pub fn verbose() -> Self {
        Self {
            indent_level: 0,
            verbose: true,
            show_line_numbers: true,
        }
    }
    
    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }
    
    /// Set line number display
    pub fn set_show_line_numbers(&mut self, show: bool) -> &mut Self {
        self.show_line_numbers = show;
        self
    }
    
    /// Print a complete MIR module
    pub fn print_module(&self, module: &MirModule) -> String {
        let mut output = String::new();
        
        // Module header
        writeln!(output, "; MIR Module: {}", module.name).unwrap();
        if let Some(ref source) = module.metadata.source_file {
            writeln!(output, "; Source: {}", source).unwrap();
        }
        writeln!(output).unwrap();
        
        // Module statistics
        if self.verbose {
            let stats = module.stats();
            writeln!(output, "; Module Statistics:").unwrap();
            writeln!(output, ";   Functions: {}", stats.function_count).unwrap();
            writeln!(output, ";   Globals: {}", stats.global_count).unwrap();
            writeln!(output, ";   Total Blocks: {}", stats.total_blocks).unwrap();
            writeln!(output, ";   Total Instructions: {}", stats.total_instructions).unwrap();
            writeln!(output, ";   Pure Functions: {}", stats.pure_functions).unwrap();
            writeln!(output).unwrap();
        }
        
        // Global constants
        if !module.globals.is_empty() {
            writeln!(output, "; Global Constants:").unwrap();
            for (name, value) in &module.globals {
                writeln!(output, "global @{} = {}", name, value).unwrap();
            }
            writeln!(output).unwrap();
        }
        
        // Functions
        for (_name, function) in &module.functions {
            output.push_str(&self.print_function(function));
            output.push('\n');
        }
        
        output
    }
    
    /// Print a single MIR function
    pub fn print_function(&self, function: &MirFunction) -> String {
        let mut output = String::new();
        
        // Function signature
        write!(output, "define {} @{}(", 
               self.format_type(&function.signature.return_type),
               function.signature.name).unwrap();
        
        for (i, param_type) in function.signature.params.iter().enumerate() {
            if i > 0 {
                write!(output, ", ").unwrap();
            }
            write!(output, "{} %{}", self.format_type(param_type), i).unwrap();
        }
        write!(output, ")").unwrap();
        
        // Effects
        if !function.signature.effects.is_pure() {
            write!(output, " effects({})", function.signature.effects).unwrap();
        }
        
        writeln!(output, " {{").unwrap();
        
        // Function statistics
        if self.verbose {
            let stats = function.stats();
            writeln!(output, "  ; Function Statistics:").unwrap();
            writeln!(output, "  ;   Blocks: {}", stats.block_count).unwrap();
            writeln!(output, "  ;   Instructions: {}", stats.instruction_count).unwrap();
            writeln!(output, "  ;   Values: {}", stats.value_count).unwrap();
            writeln!(output, "  ;   Phi Functions: {}", stats.phi_count).unwrap();
            if stats.is_pure {
                writeln!(output, "  ;   Pure: yes").unwrap();
            }
            writeln!(output).unwrap();
        }
        
        // Print blocks in order
        let mut block_ids: Vec<_> = function.blocks.keys().copied().collect();
        block_ids.sort();
        
        for (i, block_id) in block_ids.iter().enumerate() {
            if let Some(block) = function.blocks.get(block_id) {
                if i > 0 {
                    writeln!(output).unwrap();
                }
                output.push_str(&self.print_basic_block(block));
            }
        }
        
        writeln!(output, "}}").unwrap();
        
        output
    }
    
    /// Print a basic block
    pub fn print_basic_block(&self, block: &BasicBlock) -> String {
        let mut output = String::new();
        
        // Block header
        write!(output, "{}:", block.id).unwrap();
        
        // Predecessors
        if !block.predecessors.is_empty() && self.verbose {
            let preds: Vec<String> = block.predecessors.iter()
                .map(|p| format!("{}", p))
                .collect();
            write!(output, "  ; preds({})", preds.join(", ")).unwrap();
        }
        
        writeln!(output).unwrap();
        
        // Instructions
        let mut line_num = 0;
        for instruction in block.all_instructions() {
            if self.show_line_numbers {
                write!(output, "  {:3}: ", line_num).unwrap();
            } else {
                write!(output, "    ").unwrap();
            }
            
            writeln!(output, "{}", self.format_instruction(instruction)).unwrap();
            line_num += 1;
        }
        
        // Block effects (if verbose and not pure)
        if self.verbose && !block.effects.is_pure() {
            writeln!(output, "    ; effects: {}", block.effects).unwrap();
        }
        
        output
    }
    
    /// Format a single instruction
    fn format_instruction(&self, instruction: &MirInstruction) -> String {
        match instruction {
            MirInstruction::Const { dst, value } => {
                format!("{} = const {}", dst, value)
            },
            
            MirInstruction::BinOp { dst, op, lhs, rhs } => {
                format!("{} = {} {:?} {}", dst, lhs, op, rhs)
            },
            
            MirInstruction::UnaryOp { dst, op, operand } => {
                format!("{} = {:?} {}", dst, op, operand)
            },
            
            MirInstruction::Compare { dst, op, lhs, rhs } => {
                format!("{} = icmp {:?} {}, {}", dst, op, lhs, rhs)
            },
            
            MirInstruction::Load { dst, ptr } => {
                format!("{} = load {}", dst, ptr)
            },
            
            MirInstruction::Store { value, ptr } => {
                format!("store {} -> {}", value, ptr)
            },
            
            MirInstruction::Call { dst, func, args, effects: _ } => {
                let args_str = args.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                if let Some(dst) = dst {
                    format!("{} = call {}({})", dst, func, args_str)
                } else {
                    format!("call {}({})", func, args_str)
                }
            },
            
            MirInstruction::BoxCall { dst, box_val, method, args, effects: _ } => {
                let args_str = args.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                
                if let Some(dst) = dst {
                    format!("{} = call {}.{}({})", dst, box_val, method, args_str)
                } else {
                    format!("call {}.{}({})", box_val, method, args_str)
                }
            },
            
            MirInstruction::Branch { condition, then_bb, else_bb } => {
                format!("br {}, label {}, label {}", condition, then_bb, else_bb)
            },
            
            MirInstruction::Jump { target } => {
                format!("br label {}", target)
            },
            
            MirInstruction::Return { value } => {
                if let Some(value) = value {
                    format!("ret {}", value)
                } else {
                    "ret void".to_string()
                }
            },
            
            MirInstruction::Phi { dst, inputs } => {
                let inputs_str = inputs.iter()
                    .map(|(bb, val)| format!("[{}, {}]", val, bb))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} = phi {}", dst, inputs_str)
            },
            
            MirInstruction::NewBox { dst, box_type, args } => {
                let args_str = args.iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{} = new {}({})", dst, box_type, args_str)
            },
            
            MirInstruction::TypeCheck { dst, value, expected_type } => {
                format!("{} = type_check {} is {}", dst, value, expected_type)
            },
            
            MirInstruction::Cast { dst, value, target_type } => {
                format!("{} = cast {} to {:?}", dst, value, target_type)
            },
            
            MirInstruction::TypeOp { dst, op, value, ty } => {
                let op_str = match op { super::TypeOpKind::Check => "check", super::TypeOpKind::Cast => "cast" };
                format!("{} = typeop {} {} {:?}", dst, op_str, value, ty)
            },
            
            MirInstruction::ArrayGet { dst, array, index } => {
                format!("{} = {}[{}]", dst, array, index)
            },
            
            MirInstruction::ArraySet { array, index, value } => {
                format!("{}[{}] = {}", array, index, value)
            },
            
            MirInstruction::Copy { dst, src } => {
                format!("{} = copy {}", dst, src)
            },
            
            MirInstruction::Debug { value, message } => {
                format!("debug {} \"{}\"", value, message)
            },
            
            MirInstruction::Print { value, effects: _ } => {
                format!("print {}", value)
            },
            
            MirInstruction::Nop => {
                "nop".to_string()
            },
            
            // Phase 5: Control flow & exception handling
            MirInstruction::Throw { exception, effects: _ } => {
                format!("throw {}", exception)
            },
            
            MirInstruction::Catch { exception_type, exception_value, handler_bb } => {
                if let Some(ref exc_type) = exception_type {
                    format!("catch {} {} -> {}", exc_type, exception_value, handler_bb)
                } else {
                    format!("catch * {} -> {}", exception_value, handler_bb)
                }
            },
            
            MirInstruction::Safepoint => {
                "safepoint".to_string()
            },
            
            // Phase 6: Box reference operations
            MirInstruction::RefNew { dst, box_val } => {
                format!("{} = ref_new {}", dst, box_val)
            },
            
            MirInstruction::RefGet { dst, reference, field } => {
                format!("{} = ref_get {}.{}", dst, reference, field)
            },
            
            MirInstruction::RefSet { reference, field, value } => {
                format!("ref_set {}.{} = {}", reference, field, value)
            },
            
            MirInstruction::WeakNew { dst, box_val } => {
                format!("{} = weak_new {}", dst, box_val)
            },
            
            MirInstruction::WeakLoad { dst, weak_ref } => {
                format!("{} = weak_load {}", dst, weak_ref)
            },
            
            MirInstruction::BarrierRead { ptr } => {
                format!("barrier_read {}", ptr)
            },
            
            MirInstruction::BarrierWrite { ptr } => {
                format!("barrier_write {}", ptr)
            },
            
            MirInstruction::WeakRef { dst, op, value } => {
                let op_str = match op { super::WeakRefOp::New => "new", super::WeakRefOp::Load => "load" };
                format!("{} = weakref {} {}", dst, op_str, value)
            },
            
            MirInstruction::Barrier { op, ptr } => {
                let op_str = match op { super::BarrierOp::Read => "read", super::BarrierOp::Write => "write" };
                format!("barrier {} {}", op_str, ptr)
            },
            
            // Phase 7: Async/Future Operations
            MirInstruction::FutureNew { dst, value } => {
                format!("{} = future_new {}", dst, value)
            },
            
            MirInstruction::FutureSet { future, value } => {
                format!("future_set {} = {}", future, value)
            },
            
            MirInstruction::Await { dst, future } => {
                format!("{} = await {}", dst, future)
            },
            
            // Phase 9.7: External Function Calls
            MirInstruction::ExternCall { dst, iface_name, method_name, args, effects } => {
                let args_str = args.iter().map(|v| format!("{}", v)).collect::<Vec<_>>().join(", ");
                if let Some(dst) = dst {
                    format!("{} = extern_call {}.{}({}) [effects: {}]", dst, iface_name, method_name, args_str, effects)
                } else {
                    format!("extern_call {}.{}({}) [effects: {}]", iface_name, method_name, args_str, effects)
                }
            },
        }
    }
    
    /// Format a MIR type
    fn format_type(&self, mir_type: &super::MirType) -> String {
        match mir_type {
            super::MirType::Integer => "i64".to_string(),
            super::MirType::Float => "f64".to_string(),
            super::MirType::Bool => "i1".to_string(),
            super::MirType::String => "str".to_string(),
            super::MirType::Box(name) => format!("box<{}>", name),
            super::MirType::Array(elem_type) => format!("[{}]", self.format_type(elem_type)),
            super::MirType::Future(inner_type) => format!("future<{}>", self.format_type(inner_type)),
            super::MirType::Void => "void".to_string(),
            super::MirType::Unknown => "?".to_string(),
        }
    }
}

impl Default for MirPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mir::{MirModule, MirFunction, FunctionSignature, MirType, EffectMask, BasicBlockId};
    
    #[test]
    fn test_empty_module_printing() {
        let module = MirModule::new("test".to_string());
        let printer = MirPrinter::new();
        
        let output = printer.print_module(&module);
        
        assert!(output.contains("MIR Module: test"));
        assert!(!output.is_empty());
    }
    
    #[test]
    fn test_function_printing() {
        let signature = FunctionSignature {
            name: "test_func".to_string(),
            params: vec![MirType::Integer],
            return_type: MirType::Void,
            effects: EffectMask::PURE,
        };
        
        let function = MirFunction::new(signature, BasicBlockId::new(0));
        let printer = MirPrinter::new();
        
        let output = printer.print_function(&function);
        
        assert!(output.contains("define void @test_func(i64 %0)"));
        assert!(output.contains("bb0:"));
    }
    
    #[test]
    fn test_verbose_printing() {
        let module = MirModule::new("test".to_string());
        let printer = MirPrinter::verbose();
        
        let output = printer.print_module(&module);
        
        assert!(output.contains("Module Statistics"));
    }
}
