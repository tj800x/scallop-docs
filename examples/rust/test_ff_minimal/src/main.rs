// Minimal test: Can IntegrateContext use foreign functions at all?

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    test_with_interpret_context()?;
    test_with_integrate_context_correct()?;
    Ok(())
}

#[allow(dead_code)]
fn test_with_add_rule() -> Result<(), IntegrateError> {
    println!("=== Test 1: Builtin Foreign Function with add_rule() ===\n");

    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Try using a builtin foreign function (string_length exists in stdlib)
    ctx.add_relation("words(String)")?;

    println!("Attempting to add rule with $string_length...");
    ctx.add_rule(r#"result(w, len) = words(w), len = $string_length(w)"#)?;

    println!("Success! Rule accepted.");

    Ok(())
}

#[allow(dead_code)]
fn test_with_program() -> Result<(), IntegrateError> {
    println!("\n=== Test 2: Using add_program() instead ===\n");

    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    println!("Attempting to add program with $string_length...");
    ctx.add_program(r#"
        rel words = {"hello"}
        rel result(w, len) = words(w), len = $string_length(w)
        query result
    "#)?;

    println!("Success! Program accepted.");

    Ok(())
}

#[allow(dead_code)]
fn test_with_interpret_context() -> Result<(), IntegrateError> {
    println!("=== Test 3: Using InterpretContext (like tests do) ===\n");

    let prov = UnitProvenance::default();
    let mut ctx = InterpretContext::<_, RcFamily>::new(
        r#"
        rel words = {"hello", "world"}
        rel result(w, len) = words(w), len == $string_length(w)
        query result
        "#.to_string(),
        prov
    )?;

    println!("Program compiled successfully!");

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

fn test_with_integrate_context_correct() -> Result<(), IntegrateError> {
    println!("\n=== Test 4: IntegrateContext with CORRECT syntax (==) ===\n");

    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    ctx.add_program(r#"
        rel words = {"hello", "world"}
        rel result(w, len) = words(w), len == $string_length(w)
        query result
    "#)?;

    println!("Program compiled successfully!");

    ctx.run()?;
    println!("Program executed!");

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
