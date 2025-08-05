# CRUSH.md

## Build & Test Commands
Build firmware: `cargo build --release`
Flash device: `elf2uf2-rs target/thumbv6m-none-eabi/release/ass-easy-loop`
Run specific test: `cargo test --test <test_name>`
Run all tests: `cargo test`
Run validation binary: `cargo run --bin validate_<name>`

## Code Style
- Imports: Grouped std, external, then local; sorted alphabetically
- Formatting: Use rustfmt with default settings
- Types: Prefer explicit type annotations in public APIs
- Naming: snake_case for functions/variables, PascalCase for types
- Error Handling: Use custom Result type `SystemResult<T>`
- Modules: Keep related functionality co-located
- Logging: Use `logging` module with `QueueLogger`
- Safety: Mark unsafe code explicitly, prefer safe abstractions

## Architecture
- RTIC task priorities: pEMF (high), Battery (medium), LED (low)
- Shared resources managed through RTIC with compile-time safety
- Hardware interfaces abstracted through dedicated modules
- Configuration constants in `config.rs`