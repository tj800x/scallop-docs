// Test custom foreign function with InterpretContext

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::foreign_function::*;
use scallop_core::common::value::*;
use scallop_core::common::value_type::*;

#[derive(Clone)]
struct MyLen;

impl ForeignFunction for MyLen {
    fn name(&self) -> String {
        "my_len".to_string()
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

fn main() -> Result<(), IntegrateError> {
    println!("=== Test Custom Foreign Function with InterpretContext ===\n");

    let prov = UnitProvenance::default();

    // Create InterpretContext with program
    let program = r#"
        rel words = {"hello", "world"}
        rel result(w, $my_len(w)) = words(w)
        query result
    "#.to_string();

    let mut ctx = InterpretContext::<_, RcFamily>::new(program, prov)?;

    // Register function BEFORE running
    ctx.runtime_env().register_foreign_function(MyLen)?;

    println!("Program compiled, function registered");

    ctx.run()?;
    println!("Program executed!");

    let idb = ctx.idb();
    let result = idb.get_output_collection_ref("result").unwrap();
    println!("\nResults:");
    for elem in result.iter() {
        let tuple = &elem.1;
        if let (Some(Value::String(word)), Some(Value::USize(len))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  \"{}\" has length {}", word, len);
        }
    }

    Ok(())
}
