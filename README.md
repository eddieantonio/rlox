rlox
====

[![Build](https://github.com/eddieantonio/rlox/actions/workflows/rust.yml/badge.svg)](https://github.com/eddieantonio/rlox/actions/workflows/rust.yml)

My version of the Lox interpreter, implemented in  Rust.

See part III of [Crafting Interpreters][craftinginterpreters].

See also: my Java implementation, [jlox][].

[jlox]: https://github.com/eddieantonio/jlox
[craftinginterpreters]: https://craftinginterpreters.com/a-bytecode-virtual-machine.html

# Build

    cargo build

# Run

    cargo run

# Cargo Features

 - `trace_execution` — if compiled with `trace_execution`, verbose
   diagnostics are printed **to `stdout`** for every opcode executed.
   Extremely chatty — use this only for debugging.

       cargo run --features=trace_execution

 - `print_code` — if compiled with `print_code`, the Lox compiler will
   print the disassembly of the chunk it just created **to `stdout`**
   Use this to debug code generation.

       cargo run --features=print_code

# Test driven development

I hack on this iteratively by combining [`just`][just] with [`entr`][entr]:

    git ls-files | entr -c just tdd

[just]: https://github.com/casey/just
[entr]: https://eradman.com/entrproject/
