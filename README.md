# CPiler
A custom Compiler for C written entirely in Rust

## Parts
### compiler
This simply puts all the Parts together to actually compile a single C Project

### general
A Collection of general Datastructure that are used across different parts of the compiler

### tokenizer
This is responsible for taking any string of characters and turning it into Tokens that the rest
of the system can work with more easily

### preprocessor
Handles all the preprocessing needs for the compiler handling file inclusion, conditional compilation,
defines, etc.

### syntax
Handles all the Syntax parsing for the Code

### semantic
Handles all the Semantic "analysis" for the Code and builds up the final the version
of the AST for the FrontEnd containing Types

## Resources
* C-Standard: http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf
