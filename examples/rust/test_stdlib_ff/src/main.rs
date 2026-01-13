// Test stdlib foreign function with IntegrateContext using add_rule

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    println!("=== Test Stdlib FF with IntegrateContext + add_rule ===\n");

    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Try with add_program instead
    println!("Adding program with $string_length...");
    ctx.add_program(r#"
        rel words = {"hello", "world"}
        rel result(w, $string_length(w)) = words(w)
        query result
    "#)?;
    println!("Program added successfully");

    // Run
    ctx.run()?;
    println!("Program executed");

    // Query
    let result = ctx.computed_relation_ref("result").unwrap();
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
