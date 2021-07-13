# Implementation of expressions

### Expression Template

The base struct combining metadata and data of the expression, i.e. type and the actual expression

### Expression Enum

Enum that shows the selection of different expressions

### List of expressions without block

List of expressions used and how they are classified

1. [Literal expressions](#literal-expression)

List of excluded expressions or TODO

- Path expressions - for imports and referring to other modules
- Operator expressions

#### Literal expressions

Consists one of the literal forms, directly describes a number, character, string or boolean value.

```rust
"hello"; // string literal
'5';     // char literal
5;       // int literal
```
