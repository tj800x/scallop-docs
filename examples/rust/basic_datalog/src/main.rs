// Basic DataLog Example
//
// This example demonstrates:
// - Creating an IntegrateContext with UnitProvenance
// - Adding a Scallop program
// - Running the program
// - Querying results

use scallop_core::integrate::*;
use scallop_core::runtime::provenance::unit::UnitProvenance;
use scallop_core::utils::RcFamily;
use scallop_core::common::value::Value;

fn main() -> Result<(), IntegrateError> {
    println!("=== Basic DataLog Example ===\n");

    // Create context with unit provenance (standard DataLog)
    let prov = UnitProvenance::default();
    let mut ctx = IntegrateContext::<_, RcFamily>::new(prov);

    // Add a Scallop program defining edges and transitive closure
    ctx.add_program(r#"
        // Define edges as facts
        rel edge = {
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 4),
        }

        // Define path as transitive closure of edge
        rel path(a, b) = edge(a, b)
        rel path(a, c) = path(a, b), edge(b, c)

        // Query the path relation
        query path
    "#)?;

    println!("Program loaded successfully");

    // Execute the program
    ctx.run()?;
    println!("Program executed\n");

    // Get query results
    let path_relation = ctx.computed_relation_ref("path").unwrap();

    println!("Results for path relation:");
    println!("Total tuples: {}\n", path_relation.len());

    // Iterate over results and display
    for elem in path_relation.iter() {
        // elem is (tag, tuple) - access with .0 and .1
        let tuple = &elem.1;

        // GenericTuple uses Index - tuple[i] returns &GenericTuple
        // We need to get the Value inside
        if let (Some(Value::I32(from)), Some(Value::I32(to))) =
            (tuple[0].get_value(), tuple[1].get_value())
        {
            println!("  path({}, {})", from, to);
        }
    }

    println!("\n=== Example Complete ===");

    Ok(())
}
