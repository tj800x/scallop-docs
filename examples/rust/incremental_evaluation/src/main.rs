// Incremental Evaluation Example
//
// This example demonstrates:
// - Creating incremental contexts
// - Adding facts incrementally
// - Re-running after updates
// - Efficient incremental computation

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    println!("=== Incremental Evaluation Example ===\n");

    // Create INCREMENTAL context (important!)
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new_incremental(prov);
    println!("Created incremental context");

    // Declare relation type
    ctx.add_relation("edge(i32, i32)")?;

    // Define rules for transitive closure
    ctx.add_rule("path(a, b) = edge(a, b)")?;
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;
    println!("Rules defined\n");

    // === INITIAL FACTS ===
    println!("=== Round 1: Initial Facts ===");
    ctx.add_facts("edge", vec![
        (None, Tuple::from((0i32, 1i32))),
        (None, Tuple::from((1i32, 2i32))),
    ], false)?;
    println!("Added edges: (0, 1), (1, 2)");

    // Run first evaluation
    ctx.run()?;
    println!("Evaluation complete");

    // Display results
    let path = ctx.computed_relation_ref("path").unwrap();
    println!("Paths found: {}", path.len());
    for elem in path.iter() {
        let tuple = &elem.1;
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  path({}, {})", from, to);
        }
    }

    // === INCREMENT 1 ===
    println!("\n=== Round 2: Add More Edges ===");
    ctx.add_facts("edge", vec![
        (None, Tuple::from((2i32, 3i32))),
        (None, Tuple::from((3i32, 4i32))),
    ], false)?;
    println!("Added edges: (2, 3), (3, 4)");

    // Run incremental evaluation
    ctx.run()?;
    println!("Incremental evaluation complete");

    // Display NEW results
    let path = ctx.computed_relation_ref("path").unwrap();
    println!("Total paths now: {}", path.len());
    for elem in path.iter() {
        let tuple = &elem.1;
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  path({}, {})", from, to);
        }
    }

    // === INCREMENT 2 ===
    println!("\n=== Round 3: Add Shortcut Edge ===");
    ctx.add_facts("edge", vec![
        (None, Tuple::from((0i32, 3i32))),  // Shortcut
    ], false)?;
    println!("Added edge: (0, 3) [shortcut]");

    // Run incremental evaluation again
    ctx.run()?;
    println!("Incremental evaluation complete");

    // Display final results
    let path = ctx.computed_relation_ref("path").unwrap();
    println!("Final path count: {}", path.len());
    println!("(Note: count may not change - shortcut provides alternative derivation)");

    // Show sample paths
    println!("\nSample paths:");
    let mut count = 0;
    for elem in path.iter() {
        if count >= 8 { break; }
        let tuple = &elem.1;
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  path({}, {})", from, to);
            count += 1;
        }
    }
    if path.len() > 8 {
        println!("  ... and {} more", path.len() - 8);
    }

    println!("\n=== Example Complete ===");
    println!("\nKey Point:");
    println!("  - new_incremental() enables efficient updates");
    println!("  - Only affected facts are recomputed");
    println!("  - Ideal for dynamic datasets");

    Ok(())
}
