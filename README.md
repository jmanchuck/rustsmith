# rustsmith

A Rust compiler fuzzer

### Version 0.1.1

- Added the following language constructs -
  - Boolean expressions
  - Conditional statements
  - Simple borrow expressions
- Removed dependency on cargo nightly
- Simple borrow rules implemented
  - Cannot borrow a mutably borrowed variable
  - Cannot mutabaly borrow an immutably borrowed variable
  - Move not allowed on borrowed variables

### Version 0.1.0

- Generates very simple Rust programs, currently covering the following language constructs -
  - Arithmetic expressions
  - Function calls
  - Structs
  - Basic let statements and assignment statements
- Some automation for generating, building, running and differential testing generated programs
- Uses safe math wrapped functions to prevent overflow and illegal integer arithmetic
