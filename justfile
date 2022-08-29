# I do "test-driven development" by running this whenever files change
#
# Usage:
#    git ls-files | entr -c just tdd
tdd:
    cargo test
    cargo doc
    cargo run --features=trace_execution,print_code examples/current-example.lox
