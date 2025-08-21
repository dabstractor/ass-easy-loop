# AI Context

This file provides context for AI assistants working on this project.

# Important Instructions

## Build and Flash Workflow
- ALWAYS use `cargo run` as the ONLY command for building and flashing firmware
- This command handles everything automatically:
  - Building the firmware
  - Converting to appropriate format
  - Flashing to connected device
- NEVER download or use external flashing tools like uf2conv.py
- NEVER manually convert binary files to UF2 format

## Dependency Management
- NEVER add dependencies to Cargo.toml manually
- ALWAYS use `cargo add` command to add packages
- This ensures latest compatible versions are used
- Example: `cargo add rp2040-hal` (NOT manual edits to Cargo.toml)

## Code Modification Rules
- Do what has been asked; nothing more, nothing less
- NEVER create files unless they're absolutely necessary for achieving your goal
- ALWAYS prefer editing an existing file to creating a new one
- NEVER proactively create documentation files (*.md) or README files
- Only create documentation files if explicitly requested by the User