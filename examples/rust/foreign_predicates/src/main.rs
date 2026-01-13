// Foreign Predicates Example
//
// This example demonstrates:
// - Implementing the ForeignPredicate trait
// - Binding patterns (bf, ff)
// - Generating multiple results
// - External data integration

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::foreign_predicate::*;
use scallop_core::common::value::*;
use scallop_core::common::value_type::*;
use scallop_core::common::input_tag::DynamicInputTag;

// Foreign Predicate 1: Range Generator
// Generates integers from 0 to n-1
#[derive(Clone)]
pub struct Range;

impl ForeignPredicate for Range {
    fn name(&self) -> String {
        "range".to_string()
    }

    fn arity(&self) -> usize {
        2  // (n, i)
    }

    fn argument_type(&self, _i: usize) -> ValueType {
        ValueType::I32
    }

    fn num_bounded(&self) -> usize {
        1  // First argument (n) is bounded
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::I32(n) = &bounded[0] {
            (0..*n).map(|i| {
                (
                    DynamicInputTag::None,
                    vec![Value::I32(i)]  // Only return the free argument!
                )
            }).collect()
        } else {
            vec![]
        }
    }
}

// Foreign Predicate 2: String Splitter
// Splits a string into individual characters
#[derive(Clone)]
pub struct StringChars;

impl ForeignPredicate for StringChars {
    fn name(&self) -> String {
        "str_chars".to_string()
    }

    fn arity(&self) -> usize {
        2  // (string, char)
    }

    fn argument_type(&self, i: usize) -> ValueType {
        if i == 0 {
            ValueType::String
        } else {
            ValueType::Char
        }
    }

    fn num_bounded(&self) -> usize {
        1  // First argument (string) is bounded
    }

    fn evaluate(&self, bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        if let Value::String(s) = &bounded[0] {
            s.chars().map(|c| {
                (
                    DynamicInputTag::None,
                    vec![Value::Char(c)]  // Only return the free argument!
                )
            }).collect()
        } else {
            vec![]
        }
    }
}

// Foreign Predicate 3: CSV Data Generator
// Simulates loading data from a CSV file
#[derive(Clone)]
pub struct CSVData {
    data: Vec<(String, i32, String)>,
}

impl CSVData {
    pub fn new() -> Self {
        Self {
            data: vec![
                ("Alice".into(), 30, "Engineer".into()),
                ("Bob".into(), 25, "Designer".into()),
                ("Charlie".into(), 35, "Manager".into()),
                ("Diana".into(), 28, "Analyst".into()),
            ]
        }
    }
}

impl ForeignPredicate for CSVData {
    fn name(&self) -> String {
        "csv_data".to_string()
    }

    fn arity(&self) -> usize {
        3  // (name, age, role)
    }

    fn argument_type(&self, i: usize) -> ValueType {
        match i {
            0 => ValueType::String,  // name
            1 => ValueType::I32,     // age
            2 => ValueType::String,  // role
            _ => panic!("Invalid argument index"),
        }
    }

    fn num_bounded(&self) -> usize {
        0  // All free (ff pattern)
    }

    fn evaluate(&self, _bounded: &[Value]) -> Vec<(DynamicInputTag, Vec<Value>)> {
        self.data.iter().map(|(name, age, role)| {
            (
                DynamicInputTag::None,
                vec![
                    Value::String(name.clone()),
                    Value::I32(*age),
                    Value::String(role.clone()),
                ]
            )
        }).collect()
    }
}

fn main() -> Result<(), IntegrateError> {
    println!("=== Foreign Predicates Example ===\n");

    // Create context
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Register foreign predicates
    println!("Registering foreign predicates:");
    ctx.register_foreign_predicate(Range)?;
    println!("  - range(n, i) [bf pattern]");

    ctx.register_foreign_predicate(StringChars)?;
    println!("  - str_chars(s, c) [bf pattern]");

    ctx.register_foreign_predicate(CSVData::new())?;
    println!("  - csv_data(name, age, role) [ff pattern]\n");

    // Add Scallop program using foreign predicates
    ctx.add_program(r#"
        // Range example: generate sequences
        rel sizes = {3, 5, 7}
        rel sequence(n, i) = sizes(n), range(n, i)

        // String chars example: split strings
        rel words = {"hello", "world"}
        rel letters(w, c) = words(w), str_chars(w, c)

        // CSV data example: load external data
        rel employee(name, age, role) = csv_data(name, age, role)
        rel senior_employee(name) = employee(name, age, role), age >= 30

        query sequence
        query letters
        query employee
        query senior_employee
    "#)?;

    println!("Program loaded");

    // Execute
    ctx.run()?;
    println!("Program executed\n");

    // Display results
    println!("Sequences:");
    let sequence = ctx.computed_relation_ref("sequence").unwrap();
    for elem in sequence.iter() {
        let tuple = &elem.1;
        if let (Some(Value::I32(n)), Some(Value::I32(i))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            print!("  sequence({}, {})  ", n, i);
            if *i == *n - 1 { println!(); }  // Newline after each sequence
        }
    }

    println!("\nLetters:");
    let letters = ctx.computed_relation_ref("letters").unwrap();
    for elem in letters.iter() {
        let tuple = &elem.1;
        if let (Some(Value::String(word)), Some(Value::Char(c))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  \"{}\" contains '{}'", word, c);
        }
    }

    println!("\nEmployees (from CSV):");
    let employee = ctx.computed_relation_ref("employee").unwrap();
    for elem in employee.iter() {
        let tuple = &elem.1;
        if let (Some(Value::String(name)), Some(Value::I32(age)), Some(Value::String(role))) =
            (tuple[0].get_value(), tuple[1].get_value(), tuple[2].get_value())
        {
            println!("  {}, age {}, {}", name, age, role);
        }
    }

    println!("\nSenior Employees (age >= 30):");
    let senior = ctx.computed_relation_ref("senior_employee").unwrap();
    for elem in senior.iter() {
        let tuple = &elem.1;
        if let Some(Value::String(name)) = tuple[0].get_value() {
            println!("  {}", name);
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
