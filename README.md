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

# Cargo Features

 - `trace_execution` — if compiled with `trace_execution`, there will be
   verbose debugging printed for every opcode executed.

       cargo run --features=trace_execution

# Test driven development

I hack on this iteratively by combining [`just`][just] with [`entr`][entr]:

    git ls-files | entr -c just tdd

[just]: https://github.com/casey/just
[entr]: https://eradman.com/entrproject/
