// Foreign Functions Example
//
// This example demonstrates:
// - Implementing the ForeignFunction trait
// - String manipulation functions
// - Numeric operations
// - Registering and using custom functions in Scallop

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::*;
use scallop_core::common::value_type::*;

// Foreign Function 1: String Length (custom implementation)
// Note: Renamed to avoid conflict with stdlib string_length
#[derive(Clone)]
pub struct StringLen;

impl ForeignFunction for StringLen {
    fn name(&self) -> String {
        "str_len".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::USize)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let Value::String(s) = &args[0] {
            Some(Value::USize(s.len()))
        } else {
            None
        }
    }
}

// Foreign Function 2: String Uppercase
#[derive(Clone)]
pub struct StringUppercase;

impl ForeignFunction for StringUppercase {
    fn name(&self) -> String {
        "uppercase".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::String)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let Value::String(s) = &args[0] {
            Some(Value::String(s.to_uppercase()))
        } else {
            None
        }
    }
}

// Foreign Function 3: Integer Absolute Value (custom implementation)
// Note: Renamed to avoid potential conflict with stdlib abs
#[derive(Clone)]
pub struct IntAbs;

impl ForeignFunction for IntAbs {
    fn name(&self) -> String {
        "int_abs".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        1
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let Value::I32(n) = &args[0] {
            Some(Value::I32(n.abs()))
        } else {
            None
        }
    }
}

// Foreign Function 4: Maximum of Two Integers (custom implementation)
// Note: Renamed to avoid potential conflict with stdlib max
#[derive(Clone)]
pub struct IntMax;

impl ForeignFunction for IntMax {
    fn name(&self) -> String {
        "int_max".to_string()
    }

    fn num_static_arguments(&self) -> usize {
        2
    }

    fn static_argument_type(&self, _i: usize) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn return_type(&self) -> ForeignFunctionParameterType {
        ForeignFunctionParameterType::BaseType(ValueType::I32)
    }

    fn execute(&self, args: Vec<Value>) -> Option<Value> {
        if let (Value::I32(a), Value::I32(b)) = (&args[0], &args[1]) {
            Some(Value::I32(*a.max(b)))
        } else {
            None
        }
    }
}

fn main() -> Result<(), IntegrateError> {
    println!("=== Foreign Functions Example ===\n");

    // Create context
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Register foreign functions
    println!("Registering foreign functions:");
    ctx.register_foreign_function(StringLen)?;
    println!("  - str_len(String) -> USize");

    ctx.register_foreign_function(StringUppercase)?;
    println!("  - uppercase(String) -> String");

    ctx.register_foreign_function(IntAbs)?;
    println!("  - int_abs(i32) -> i32");

    ctx.register_foreign_function(IntMax)?;
    println!("  - int_max(i32, i32) -> i32\n");

    // Add program with foreign functions
    // NOTE: Must use add_program(), not add_rule() for foreign functions
    ctx.add_program(r#"
        rel words = {"hello", "world", "scallop"}
        rel word_length(w, $str_len(w)) = words(w)
        rel word_upper(w, $uppercase(w)) = words(w)

        rel numbers = {-5, 10, -3, 7}
        rel absolute(n, $int_abs(n)) = numbers(n)
        rel pair_max(a, b, $int_max(a, b)) = numbers(a), numbers(b), a < b

        query word_length
        query word_upper
        query absolute
        query pair_max
    "#)?;

    println!("Program loaded");

    // Execute
    ctx.run()?;
    println!("Program executed\n");

    // Display results
    println!("String Lengths:");
    let word_length = ctx.computed_relation_ref("word_length").unwrap();
    for elem in word_length.iter() {
        let tuple = &elem.1;
        if let (Some(Value::String(word)), Some(Value::USize(len))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  \"{}\" has length {}", word, len);
        }
    }

    println!("\nUppercase Conversions:");
    let word_upper = ctx.computed_relation_ref("word_upper").unwrap();
    for elem in word_upper.iter() {
        let tuple = &elem.1;
        if let (Some(Value::String(word)), Some(Value::String(upper))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  \"{}\" -> \"{}\"", word, upper);
        }
    }

    println!("\nAbsolute Values:");
    let absolute = ctx.computed_relation_ref("absolute").unwrap();
    for elem in absolute.iter() {
        let tuple = &elem.1;
        if let (Some(Value::I32(n)), Some(Value::I32(abs_n))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  abs({}) = {}", n, abs_n);
        }
    }

    println!("\nPair Maximums (sample):");
    let pair_max = ctx.computed_relation_ref("pair_max").unwrap();
    let mut count = 0;
    for elem in pair_max.iter() {
        if count >= 5 { break; }  // Show first 5
        let tuple = &elem.1;
        if let (Some(Value::I32(a)), Some(Value::I32(b)), Some(Value::I32(m))) =
            (tuple[0].get_value(), tuple[1].get_value(), tuple[2].get_value())
        {
            println!("  max({}, {}) = {}", a, b, m);
            count += 1;
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
