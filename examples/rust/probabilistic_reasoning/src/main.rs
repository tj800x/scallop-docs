// Probabilistic Reasoning Example
//
// This example demonstrates:
// - Using MinMaxProbProvenance for probabilistic reasoning
// - Adding facts with probabilities
// - Interpreting confidence scores
// - Probabilistic transitive closure

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::min_max_prob::MinMaxProbProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    println!("=== Probabilistic Reasoning Example ===\n");

    // Create context with min-max probability provenance
    let prov = MinMaxProbProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    println!("Using MinMaxProbProvenance:");
    println!("  - add(p1, p2) = max(p1, p2)  // Best alternative");
    println!("  - mult(p1, p2) = min(p1, p2) // Weakest link\n");

    // Declare relation type
    ctx.add_relation("edge(i32, i32)")?;

    // Add facts with probabilities
    println!("Adding probabilistic edges:");
    ctx.add_facts("edge", vec![
        (Some(0.9.into()), Tuple::from((0i32, 1i32))),
        (Some(0.8.into()), Tuple::from((1i32, 2i32))),
        (Some(0.7.into()), Tuple::from((2i32, 3i32))),
        (Some(0.6.into()), Tuple::from((0i32, 2i32))),  // Shortcut
    ], false)?;

    println!("  edge(0, 1) with probability 0.9");
    println!("  edge(1, 2) with probability 0.8");
    println!("  edge(2, 3) with probability 0.7");
    println!("  edge(0, 2) with probability 0.6 (shortcut)\n");

    // Define transitive closure
    ctx.add_rule("path(a, b) = edge(a, b)")?;
    ctx.add_rule("path(a, c) = path(a, b), edge(b, c)")?;

    println!("Rules defined:");
    println!("  path(a, b) = edge(a, b)");
    println!("  path(a, c) = path(a, b), edge(b, c)\n");

    // Execute
    println!("Executing...");
    ctx.run()?;
    println!("Done\n");

    // Query paths with probabilities
    let path = ctx.computed_relation_ref("path").unwrap();

    println!("Probabilistic Paths:");
    println!("(Showing how probabilities propagate)\n");

    for elem in path.iter() {
        // elem is (tag, tuple) - access with .0 and .1
        let tuple = &elem.1;

        // GenericTuple uses Index - tuple[i] returns &GenericTuple
        // We need to get the Value inside
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            print!("  path({}, {}) with confidence: {:.2}", from, to, elem.0);

            // Explain some derivations
            match (from, to) {
                (0, 1) => println!("  // Direct edge: 0.9"),
                (1, 2) => println!("  // Direct edge: 0.8"),
                (2, 3) => println!("  // Direct edge: 0.7"),
                (0, 2) => println!("  // max(0.6 direct, min(0.9, 0.8) via 1) = max(0.6, 0.8) = 0.8"),
                (1, 3) => println!("  // min(0.8, 0.7) = 0.7"),
                (0, 3) => println!("  // Best path via 1,2: min(0.9, 0.8, 0.7) = 0.7"),
                _ => println!(),
            }
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
