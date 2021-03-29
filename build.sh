#!/bin/sh
# Build code using nightly compiler.
# Default target is set in .cargo/config.toml.
exec cargo +nightly-2021-01-07 build --release

# LLVM ERROR: Not supported instr: <MCInst 296 <MCOperand Reg:1> <MCOperand Imm:15> <MCOperand Reg:39>>
#exec cargo +nightly-x86_64-unknown-linux-gnu build --release "$@"
