# Design Document

## Overview

This design establishes a comprehensive organizational structure for the battery-operated pEMF device project, transforming the current cluttered root directory into a well-organized, maintainable codebase. The reorganization follows industry-standard conventions while preserving all existing functionality and ensuring seamless operation of the pEMF generation system.

## Architecture

### Organizational Principles

1. **Separation of Concerns**: Different types of files are organized into dedicated directories based on their purpose
2. **Logical Grouping**: Related files are co-located to improve discoverability and maintenance
3. **Standard Conventions**: Following Rust/embedded project conventions and industry best practices
4. **Functional Preservation**: All existing functionality must remain intact with updated path references
5. **Future-Proof Structure**: Organization supports future growth and additional features

### Directory Structure Design

```
project-root/
├── src/                          # Core Rust source code (unchanged)
├── tests/                        # Rust integration tests (unchanged)
├── docs/                         # All documentation
│   ├── setup/                    # Setup and installation guides
│   ├── hardware/                 # Hardware-related documentation
│   ├── api/                      # API and usage documentation
│   ├── troubleshooting/          # Troubleshooting guides
│   └── development/              # Development environment docs
├── scripts/                      # All executable scripts
│   ├── validation/               # Hardware validation scripts
│   ├── bootloader/               # Bootloader-related scripts
│   ├── testing/                  # Test execution scripts
│   └── utilities/                # General utility scripts
├── test_framework/               # Complete test framework (unchanged location)
├── artifacts/                    # Generated files and outputs
│   ├── test_results/             # Test output files
│   ├── firmware/                 # Generated firmware files
│   └── logs/                     # Log files
├── .kiro/                        # Kiro configuration (unchanged)
├── target/                       # Rust build artifacts (unchanged)
├── Cargo.toml                    # Project configuration (unchanged)
├── README.md                     # Main project documentation
├── .gitignore                    # Git ignore rules
└── build.rs                      # Build script (unchanged)
```

## Components and Interfaces

### Documentation Organization (`docs/`)

**Purpose**: Centralize all project documentation with logical categorization

**Structure**:
- `setup/`: Installation, wiring, and initial setup guides
- `hardware/`: Hardware specifications, validation, and troubleshooting
- `api/`: API documentation, usage examples, and integration guides
- `troubleshooting/`: Problem diagnosis and resolution guides
- `development/`: Development environment and contribution guides

**File Mappings**:
- Setup guides → `docs/setup/`
- Hardware documentation → `docs/hardware/`
- API and usage docs → `docs/api/`
- Troubleshooting guides → `docs/troubleshooting/`
- Development docs → `docs/development/`

### Script Organization (`scripts/`)

**Purpose**: Organize all executable scripts by functional category

**Structure**:
- `validation/`: Hardware validation and testing scripts
- `bootloader/`: Bootloader flashing, debugging, and diagnostic scripts
- `testing/`: Test execution and validation scripts
- `utilities/`: General-purpose utility scripts

**Key Considerations**:
- Preserve executable permissions
- Update internal path references
- Maintain import dependencies
- Ensure Python path compatibility

### Artifact Management (`artifacts/`)

**Purpose**: Centralize all generated files and build outputs

**Structure**:
- `test_results/`: HTML, CSV, and log files from test runs
- `firmware/`: Generated UF2 and binary files
- `logs/`: Runtime logs and debug outputs

**Benefits**:
- Clear separation of source and generated content
- Easy cleanup of temporary files
- Improved .gitignore management

## Data Models

### File Classification System

**Core Source Files** (remain in place):
- `src/` - Rust source code
- `tests/` - Rust integration tests
- `Cargo.toml`, `build.rs`, `memory.x` - Build configuration

**Documentation Files** (move to `docs/`):
- Setup guides: `*SETUP*.md`, `*INSTALLATION*.md`
- Hardware docs: `*HARDWARE*.md`, `*WIRING*.md`
- API docs: `*API*.md`, `*USAGE*.md`
- Troubleshooting: `*TROUBLESHOOTING*.md`
- Development: `*DEVELOPMENT*.md`
- Implementation summaries and task documentation

**Script Files** (move to `scripts/`):
- Validation: `validate_*.py`, `run_hardware_validation.py`
- Bootloader: `*bootloader*.py`, `debug_bootloader*.py`
- Testing: `test_*.py`, `*test*.py`
- Utilities: `hidlog.py`, general-purpose scripts

**Artifact Files** (move to `artifacts/`):
- Test results: `*.html`, `*.csv`, `Test Suite_*.csv`
- Firmware: `*.uf2`, compiled binaries
- Logs: `*.log`, debug outputs

### Path Reference Updates

**Python Import Updates**:
- Update relative imports in moved scripts
- Ensure Python path includes new script locations
- Maintain compatibility with existing workflows

**Documentation Cross-References**:
- Update internal links between documentation files
- Preserve external references and URLs
- Maintain README.md as primary entry point

## Error Handling

### Migration Safety Measures

1. **Backup Strategy**: Create backup of current state before reorganization
2. **Incremental Migration**: Move files in logical groups with validation
3. **Path Validation**: Verify all path references after each move
4. **Functionality Testing**: Run tests after each major reorganization step

### Rollback Procedures

1. **Git Integration**: Use git to track all moves and changes
2. **Checkpoint Creation**: Create commits at each major reorganization step
3. **Automated Rollback**: Script to reverse changes if issues are detected
4. **Manual Recovery**: Document manual steps for emergency recovery

### Validation Checks

1. **Build Verification**: Ensure Rust project builds successfully
2. **Test Execution**: Run all unit and integration tests
3. **Script Functionality**: Verify all Python scripts execute correctly
4. **HID Communication**: Test USB HID logging functionality
5. **Bootloader Operations**: Validate bootloader flashing and debugging
6. **pEMF Generation**: Verify core pEMF functionality remains intact

## Testing Strategy

### Pre-Reorganization Testing

1. **Baseline Establishment**:
   - Run complete test suite to establish baseline
   - Document current functionality and performance
   - Capture HID communication logs
   - Test bootloader operations

2. **Dependency Mapping**:
   - Identify all file dependencies and imports
   - Map cross-references between files
   - Document external tool dependencies

### During Reorganization Testing

1. **Incremental Validation**:
   - Test after each major file move
   - Verify imports and path references
   - Run subset of tests relevant to moved files
   - Check for broken functionality immediately

2. **Path Reference Validation**:
   - Automated scanning for broken references
   - Update and test import statements
   - Verify relative path calculations
   - Test script execution from new locations

### Post-Reorganization Testing

1. **Complete System Validation**:
   - Full Rust test suite execution
   - Python script functionality testing
   - HID communication validation
   - Bootloader operation verification

2. **Performance Verification**:
   - Ensure no performance degradation
   - Validate timing-critical operations
   - Test real-time system behavior
   - Verify embedded system functionality

3. **Integration Testing**:
   - Test complete workflows end-to-end
   - Validate CI/CD pipeline compatibility
   - Test development environment setup
   - Verify documentation accuracy

### Automated Testing Integration

1. **Test Framework Preservation**:
   - Maintain existing test_framework/ structure
   - Update any hardcoded paths in test configurations
   - Ensure CI/CD integration remains functional
   - Preserve test result generation

2. **Continuous Validation**:
   - Automated tests for path reference integrity
   - Regular validation of moved file functionality
   - Integration with existing CI/CD pipelines
   - Automated rollback triggers for failures

## Implementation Considerations

### File Move Strategy

1. **Logical Grouping**: Move related files together to minimize broken references
2. **Dependency Order**: Move files in dependency order (dependencies first)
3. **Atomic Operations**: Use git mv for proper version control tracking
4. **Immediate Testing**: Test functionality immediately after each move

### Path Reference Management

1. **Relative Path Preservation**: Maintain relative relationships where possible
2. **Absolute Path Updates**: Update hardcoded absolute paths
3. **Environment Variables**: Use environment variables for flexible paths
4. **Configuration Updates**: Update any configuration files with new paths

### Compatibility Maintenance

1. **Backward Compatibility**: Provide temporary symlinks if needed
2. **Documentation Updates**: Update all documentation with new paths
3. **Script Robustness**: Add path detection logic to scripts
4. **Error Messages**: Improve error messages to guide users to new locations

This design ensures a systematic, safe reorganization that improves project maintainability while preserving all existing functionality and ensuring the embedded system continues to operate correctly.