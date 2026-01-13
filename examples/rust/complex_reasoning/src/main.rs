// Complex Reasoning Example
//
// This example demonstrates:
// - TopKProofsProvenance for proof tracking
// - Understanding how facts combine
// - Weighted Model Counting (WMC)
// - Advanced provenance usage

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::top_k_proofs::TopKProofsProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::tuple::Tuple;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    println!("=== Complex Reasoning Example ===\n");

    // Create context with top-k proofs provenance
    let prov = TopKProofsProvenance::<RcFamily>::new(3, false);  // Track top-3 proofs, no WMC disjunctions
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    println!("Using TopKProofsProvenance:");
    println!("  - Tracks derivation proofs (how facts are derived)");
    println!("  - Computes probabilities via Weighted Model Counting");
    println!("  - Keeps top-K most probable proofs\n");

    // Declare relation
    ctx.add_relation("edge(i32, i32)")?;

    // Add probabilistic edges
    println!("Adding probabilistic edges:");
    ctx.add_facts("edge", vec![
        (Some((0.8, 0).into()), Tuple::from((0i32, 1i32))),  // (prob, fact_id)
        (Some((0.9, 1).into()), Tuple::from((1i32, 2i32))),
        (Some((0.7, 2).into()), Tuple::from((2i32, 3i32))),
        (Some((0.6, 3).into()), Tuple::from((0i32, 2i32))),  // Shortcut
    ], false)?;

    println!("  edge(0, 1) with prob 0.8 [fact_id: 0]");
    println!("  edge(1, 2) with prob 0.9 [fact_id: 1]");
    println!("  edge(2, 3) with prob 0.7 [fact_id: 2]");
    println!("  edge(0, 2) with prob 0.6 [fact_id: 3] (shortcut)\n");

    // Define multi-step reasoning
    ctx.add_program(r#"
        // Basic path
        rel path(a, b) = edge(a, b)

        // Transitive closure
        rel path(a, c) = path(a, b), edge(b, c)

        // Multi-hop paths (3+ steps)
        rel long_path(a, d) = path(a, b), path(b, c), path(c, d)

        query path
        query long_path
    "#)?;

    println!("Rules defined:");
    println!("  path(a, b) = edge(a, b)");
    println!("  path(a, c) = path(a, b), edge(b, c)");
    println!("  long_path(a, d) = path(a, b), path(b, c), path(c, d)\n");

    // Execute
    println!("Executing...");
    ctx.run()?;
    println!("Done\n");

    // Query paths
    println!("=== Paths with Probabilities ===");
    let path = ctx.computed_relation_ref("path").unwrap();

    for elem in path.iter() {
        let tuple = &elem.1;
        let tag = elem.0;
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            print!("path({}, {}) = {:.4}", from, to, tag);

            // Explain interesting cases
            match (from, to) {
                (0, 2) => {
                    println!("  // Two derivations:");
                    println!("                           //   1. Direct: 0.6");
                    println!("                           //   2. Via 1: 0.8 × 0.9 = 0.72");
                    println!("                           //   Combined via WMC: ~0.85");
                }
                (0, 3) => {
                    println!("  // Multiple paths:");
                    println!("                           //   Best: via 1,2 = 0.8 × 0.9 × 0.7");
                }
                _ => println!(),
            }
        }
    }

    // Query long paths
    println!("\n=== Long Paths (3+ hops) ===");
    let long_path = ctx.computed_relation_ref("long_path").unwrap();

    if long_path.len() > 0 {
        println!("Found {} long paths:", long_path.len());
        for elem in long_path.iter() {
            let tuple = &elem.1;
            let tag = elem.0;
            if let (Some(Value::I32(from)), Some(Value::I32(to))) =
                (tuple[0].get_value(), tuple[1].get_value())
            {
                println!("  long_path({}, {}) = {:.4}", from, to, tag);
            }
        }
    } else {
        println!("No long paths found (graph too small)");
    }

    println!("\n=== Understanding Proofs ===");
    println!("\nTopKProofsProvenance tracks:");
    println!("  1. Which facts were used in each derivation");
    println!("  2. Multiple alternative derivations (proofs)");
    println!("  3. Combines them using Weighted Model Counting\n");

    println!("Example for path(0, 2):");
    println!("  Proof 1: Uses fact_id 3 (direct edge 0→2)");
    println!("    Probability: 0.6");
    println!("  Proof 2: Uses fact_ids {{0, 1}} (edges 0→1, 1→2)");
    println!("    Probability: 0.8 × 0.9 = 0.72");
    println!("  Combined (inclusion-exclusion):");
    println!("    0.6 + 0.72 - (0.6 × 0.72) ≈ 0.888");

    println!("\n=== Example Complete ===");
    println!("\nKey Takeaways:");
    println!("  - Proofs track derivation history");
    println!("  - WMC computes exact probabilities from proofs");
    println!("  - TopK keeps most probable explanations");
    println!("  - Useful for explainability and debugging");

    Ok(())
}
