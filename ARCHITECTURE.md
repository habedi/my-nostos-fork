# Nostos Architecture

This document describes the current implementation architecture of Nostos.

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         Source Code                              │
│                        (.nos files)                              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      crates/compiler                             │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────────────┐  │
│  │  Lexer  │ → │ Parser  │ → │  Types  │ → │ Code Generator  │  │
│  └─────────┘   └─────────┘   └─────────┘   └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                        Bytecode                                  │
│               (register-based instructions)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              ▼                               ▼
┌──────────────────────┐         ┌──────────────────────┐
│     crates/vm        │         │     crates/jit       │
│    (Interpreter)     │         │    (Cranelift)       │
│                      │◄───────►│                      │
│  - Execute bytecode  │         │  - Compile hot paths │
│  - GC management     │         │  - Native code gen   │
│  - Value handling    │         │                      │
└──────────────────────┘         └──────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     crates/scheduler                             │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │                    Tokio Runtime                         │    │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐        │    │
│  │  │Worker 1 │ │Worker 2 │ │Worker 3 │ │Worker N │        │    │
│  │  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘        │    │
│  │       │           │           │           │              │    │
│  │       ▼           ▼           ▼           ▼              │    │
│  │  ┌─────────────────────────────────────────────────┐    │    │
│  │  │           Lightweight Processes (~2KB)           │    │    │
│  │  │  ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐ ┌─────┐       │    │    │
│  │  │  │ P1  │ │ P2  │ │ P3  │ │ ... │ │ Pn  │       │    │    │
│  │  │  └─────┘ └─────┘ └─────┘ └─────┘ └─────┘       │    │    │
│  │  └─────────────────────────────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Structure

### `crates/compiler`

The front-end: parsing and compilation to bytecode.

| Module | Responsibility |
|--------|----------------|
| `lexer.rs` | Tokenization |
| `parser.rs` | AST construction |
| `ast.rs` | AST node definitions |
| `types.rs` | Type definitions |
| `infer.rs` | Hindley-Milner type inference |
| `compile.rs` | Bytecode generation |

**Key features:**
- Full Hindley-Milner type inference with constraints
- Trait resolution and monomorphization
- Pattern match compilation
- Module system with visibility

### `crates/vm`

The runtime: bytecode interpreter and memory management.

| Module | Responsibility |
|--------|----------------|
| `value.rs` | Value representation, instructions |
| `runtime.rs` | Bytecode interpreter loop |
| `gc.rs` | Tracing garbage collector |
| `builtins.rs` | Built-in functions |

**Value representation:**
```
Immediate (unboxed):     Heap-allocated (GC-managed):
├── Unit                 ├── String
├── Bool                 ├── List (persistent)
├── Int (i64)            ├── Map (persistent)
├── Float (f64)          ├── Set (persistent)
├── Char                 ├── Record
├── Pid                  ├── Variant
└── MVar reference       ├── Closure
                         ├── Typed Arrays
                         └── Buffer
```

**Instruction set:**
- Register-based (not stack-based)
- ~100 instructions
- Typed operations for optimization
- Concurrency primitives (Spawn, Send, Receive)

### `crates/jit`

JIT compilation using Cranelift.

- Compiles hot functions to native code
- Triggered by call count threshold
- Supports numeric operations, control flow
- Falls back to interpreter for complex cases

### `crates/scheduler`

Process scheduler built on Tokio.

| Component | Responsibility |
|-----------|----------------|
| Scheduler | Process lifecycle, run queue |
| Process | Mailbox, state, heap reference |
| MVar | Synchronized mutable containers |
| I/O integration | Non-blocking file, network, timers |

**Concurrency model:**
- Erlang-style lightweight processes
- Message passing (deep copy between heaps)
- Cooperative scheduling with reduction counting
- All I/O is async via Tokio

### `crates/repl`

Interactive environment and editor support.

| Module | Responsibility |
|--------|----------------|
| `engine.rs` | Compilation, execution, state management |
| `autocomplete.rs` | Context-aware completions |
| `tui.rs` | Terminal UI with editor |

**Features:**
- Multi-file project support
- Live error checking
- Autocomplete with type info
- Integrated debugger and profiler

### `crates/lsp`

Language Server Protocol implementation.

- Real-time diagnostics
- Go to definition
- Hover information
- Autocomplete
- Used by VS Code extension

### `crates/cli`

Command-line interface.

```
nostos                    # Start REPL
nostos file.nos           # Run file
nostos --profile file.nos # Run with profiling
nostos --debug file.nos   # Run with debug info
```

## Data Structures

### Persistent Collections

Lists, Maps, and Sets are immutable with structural sharing:

```
Original:  [1, 2, 3, 4, 5]
                │
After prepend 0:
           [0] → [1, 2, 3, 4, 5]  (shares tail)
```

- O(1) prepend to lists
- O(log n) map/set operations
- Safe concurrent access (no locks needed)
- Memory efficient through sharing

### Typed Arrays

For numeric computation:

```
Float64Array  - 64-bit floats, contiguous memory
Int64Array    - 64-bit integers, contiguous memory
Float32Array  - 32-bit floats (GPU-compatible)
```

- O(1) index access
- Cache-friendly layout
- Direct FFI interop

## Garbage Collection

Tracing mark-and-sweep collector:

1. **Roots**: Registers, stack, globals, process mailboxes
2. **Mark**: Trace from roots, mark reachable objects
3. **Sweep**: Free unmarked objects

**Triggers:**
- Allocation threshold exceeded
- Explicit `gc()` call
- Process idle

## Type System

### Inference

Hindley-Milner with extensions:

```
1. Generate fresh type variables
2. Collect constraints from expressions
3. Unify constraints
4. Apply substitution
5. Generalize remaining variables
```

### Traits

- Nominal trait matching
- Supertraits (trait inheritance)
- Conditional implementations (`when T: Trait`)
- Operator overloading via traits

### Monomorphization

Generic functions are specialized at call sites:

```
identity[T](x: T) = x

identity(42)      → identity_Int(x: Int)
identity("hello") → identity_String(x: String)
```

## Extension System

Native extensions via Rust crates:

```toml
# nostos.toml
[extensions]
glam = { git = "https://github.com/pegesund/nostos-glam" }
```

Extensions provide:
- Type definitions
- Function bindings
- Automatic type marshaling

## File Structure

```
nostos/
├── crates/
│   ├── compiler/    # Front-end
│   ├── vm/          # Runtime
│   ├── jit/         # JIT compiler
│   ├── scheduler/   # Process scheduler
│   ├── repl/        # Interactive environment
│   ├── lsp/         # Language server
│   └── cli/         # Command line
├── stdlib/          # Standard library (.nos)
├── editors/
│   └── vscode/      # VS Code extension
├── tests/           # Test suite
├── examples/        # Example programs
├── bench/           # Benchmarks
└── docs/            # Documentation
```

## Performance Characteristics

| Operation | Complexity | Notes |
|-----------|------------|-------|
| Function call | O(1) | Register-based, no stack manipulation |
| List prepend | O(1) | Persistent structure |
| List index | O(n) | Linked structure |
| Map lookup | O(log n) | Balanced tree |
| Array index | O(1) | Contiguous memory |
| Process spawn | O(1) | Lightweight (~2KB) |
| Message send | O(n) | Deep copy of message |

## Future Directions

- [ ] Incremental GC for lower pause times
- [ ] More aggressive JIT optimization
- [ ] Native code AOT compilation
- [ ] Distributed processes across nodes
