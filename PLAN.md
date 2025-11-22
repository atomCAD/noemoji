# Plan: Unicode Compliance Linter (noemoji-rs)

A Rust-based command-line utility that enforces ASCII-first typography style guides in code and documentation, flagging decorative Unicode characters that detract from professional, clean code appearance. Prioritizes consistent style over avoiding false positives.

## Outcomes

- Detect decorative Unicode characters that violate ASCII-first style guidelines
  - Success criteria:
    - Identifies emojis (all Unicode emoji ranges)
    - Detects fancy arrows (→, ←, ⇒, ↑, ↓ etc.) vs ASCII equivalents
    - Finds decorative checkmarks/crosses (✓, ✔, ✗, ✘) vs bracket notation
    - Spots box drawing characters (┌, ─, │, └) vs ASCII art
    - Catches mathematical symbols (≤, ≥, ≠) vs ASCII operators
    - Identifies superscripts/subscripts (²³⁴, ₁₂₃) vs caret/underscore notation
    - Finds fraction characters (½, ¾) vs slash notation
    - Detects other decorative symbols (★, ●, ♦)
  - Performance constraint: Process large files efficiently
  - Principle: Enforce consistent ASCII-first typography style

- Provide helpful ASCII replacement suggestions for violations
  - Success criteria:
    - Maps arrows to ASCII equivalents (→ becomes "->")
    - Suggests bracket notation for checkmarks (✓ becomes "[x]")
    - Offers ASCII art alternatives for box drawing
    - Provides programming operators for math symbols (≤ becomes "<=")
    - Converts super/subscripts to caret/underscore notation (² becomes "^2")
    - Suggests fractional notation (½ becomes "1/2")
  - Constraint: Suggestions must maintain semantic meaning
  - Principle: ASCII-first approach for professional code appearance

- Support flexible configuration via TOML files
  - Success criteria:
    - Loads .noemoji.toml from current directory or parent directories
    - Supports allow/deny patterns for fine-grained control
    - Enables/disables specific character categories
    - Validates configuration with clear error messages
  - Constraint: Configuration must be discoverable and hierarchical
  - Principle: Sensible defaults with override capability

- Handle diverse file processing scenarios with Unix-like output
  - Success criteria:
    - Processes individual files and glob patterns (**/*.rs)
    - Supports recursive directory traversal with filtering
    - Respects .gitignore files when processing directories (--no-ignore to bypass)
    - Processes any valid UTF-8 file by default (binary detection skips non-text)
    - Optional extension filtering for discovered files (via config)
    - Warns when explicit files are skipped (binary/non-UTF-8)
    - Silently skips filtered discovered files (no warnings)
    - Outputs violations line-by-line as they're found (file:line:column format)
    - Sends processing errors to stderr
  - Performance constraint: Efficiently processes large codebases
  - Principle: Honor user intent for explicit files (warn if skipped)
  - Principle: Standard Unix tool behavior and conventions

- Provide multiple output formats for different contexts
  - Success criteria:
    - Human-readable text format displays violations with file:line:column and suggestions
    - JSON format provides structured data parseable by standard tools (jq, etc.)
    - GitHub Actions format creates ::error:: annotations visible in CI runs
    - SARIF format enables integration with security and code analysis tools
    - Checkstyle XML format supports build system integration (Jenkins, Maven, Gradle)
    - CSV/TSV formats enable data analysis and reporting workflows
    - Verbose mode includes Unicode analysis (block names, code points, encoding details)
    - Quiet mode produces no output except errors, returns correct exit codes
    - All formats provide identical violation information (no data loss across formats)
    - Format validation provides helpful error messages for invalid options
    - Output streams correctly route violations to stdout and errors to stderr
    - Terminal features (color, width) adapt gracefully in different environments
    - Custom format templates enable project-specific output styles
    - Format capability negotiation supports feature detection (color, streaming, etc.)
  - Constraint: Format selection via --format flag with validation
  - Constraint: Formatters implement common OutputFormatter trait
  - Constraint: Exit codes follow standard conventions (0=success, 1=violations, 2=errors)
  - Constraint: All output must be streaming-capable for large datasets
  - Principle: Machine-readable and human-friendly outputs serve different use cases
  - Principle: Output format extensibility through trait-based plugin architecture
  - Performance: Streaming output for large violation counts (>10k violations)
  - Performance: Memory usage remains constant regardless of violation count
  - Security: Proper escaping prevents injection in structured formats (JSON, XML, SARIF)
  - Quality: Format consistency validated through comprehensive test matrix

- Offer automatic fixing capabilities with safety measures
  - Success criteria:
    - --fix flag applies ASCII replacements automatically
    - Creates .bak backup files before modification
    - --dry-run previews changes without modification
    - --no-backup option for CI environments
    - Reports all changes made with before/after context
  - Security principle: Never modify files without explicit user consent
  - Safety constraint: Always preserve original content through backups

- Respect limited exceptions for essential non-ASCII content
  - Success criteria:
    - Allows latin diacritics (naïve, résumé, café)
    - Permits non-English languages and international content (世界, Москва)
    - Accepts currency symbols used in financial data (e.g. £, ¥, €, ₹, ₽, ₩)
    - Allows minimal technical symbols without ASCII equivalents (e.g. °, ∞) but flags symbols with ASCII alternatives (±→+/-, ×→*, ÷→/)
    - Permits legal/formal symbols used in contracts/documentation (©, ®, ™, §, ¶)
    - Otherwise flags decorative Unicode that has ASCII equivalents
  - Principle: Professional content with legitimate Unicode vs decorative flourishes
  - Constraint: Unicode substitutable with ASCII should be flagged (including math operators like ±, ×, ÷, ≤, ≥, ≠)

- Make optimal dependency choices to meet project requirements
  - Success criteria:
    - CLI parsing uses lexopt for lightweight, standards-compliant argument handling
    - Unicode processing uses minimal, focused libraries for specific tasks
    - Configuration uses serde + toml for robust TOML parsing with validation
    - File processing uses walkdir + ignore for efficient directory traversal
    - Output formatting uses appropriate libraries for each target format
    - Dependencies are minimal, well-maintained, and have clear licensing
  - Principle: Prefer smaller, focused libraries over large frameworks
  - Constraint: Dependencies should preferably be actively maintained with recent updates
  - Performance: Minimize compilation time and binary size through careful selection

- Establish open source policy and licensing framework
  - Success criteria:
    - Project licensed under Mozilla Public License 2.0 (MPL-2.0)
    - All source code files include appropriate MPL-2.0 headers
    - LICENSE file contains full MPL-2.0 license text
    - README documents licensing terms and contributor requirements
    - Dependency licenses compatible with MPL-2.0 (no GPL conflicts)
    - Contribution guidelines specify MPL-2.0 licensing requirements
  - Principle: Copyleft licensing that allows proprietary combination while protecting improvements
  - Constraint: All dependencies must be compatible with MPL-2.0 requirements
  - Legal: MPL-2.0 allows commercial use while ensuring source availability for modifications

- Command-line interface follows Unix philosophy for intuitive usage
  - Success criteria:
    - Tool accepts files as arguments or stdin for pipeline integration
    - Exit codes follow conventions (0=success, 1=violations, 2=error)
    - Help text is comprehensive with usage examples
    - Error messages are clear and actionable
    - Flags are intuitive and follow common CLI conventions
    - Progress indicators for long operations (when outputting to TTY)
    - Quiet mode suppresses non-essential output
    - Verbose mode provides detailed diagnostic information
  - Principle: Do one thing well, composable with other Unix tools
  - Principle: Human-readable output by default, machine-readable via flags
  - Constraint: CLI argument parsing uses lexopt for standards compliance

- **MVP Release**: Deliver a working linter with core detection and reporting
  - Success criteria:
    - Detects all major decorative Unicode categories (emojis, fancy arrows, checkmarks, box drawing, math symbols)
    - Provides ASCII replacement suggestions for each violation
    - Processes individual files from command-line arguments
    - Outputs violations in human-readable text format (file:line:column with suggestions)
    - Respects essential exceptions (diacritics, proper typography, currency symbols, non-English text)
    - Includes --help and --version flags
    - Licensed under MPL-2.0 with proper headers
    - Exit codes follow conventions (0=success, 1=violations, 2=error)
  - Performance constraint: Efficiently processes files up to 10MB
  - Principle: Sensible defaults, no configuration required for basic usage
  - Deferred to post-MVP: TOML configuration, directory traversal, multiple output formats, automatic fixing

## Tasks

### Project Infrastructure

- [x] Initialize basic Rust project structure
  - Run `cargo init --name noemoji` to create project scaffold
  - Verify Cargo.toml exists with correct package name "noemoji"
  - Verify src/main.rs exists with default content
  - Create src/lib.rs with basic module structure
  - Verify `cargo build` succeeds without errors

- [x] Create .gitignore for Rust project
  - Create .gitignore with Rust-specific patterns (target/)
  - Add editor-specific patterns (.vscode/, .idea/, *.swp)
  - Add OS-specific patterns (.DS_Store, Thumbs.db)
  - Run `cargo build` and verify git status excludes target/ directory

- [x] Configure project metadata in Cargo.toml
  - Set description to "A Rust CLI tool for enforcing ASCII-first typography guidelines"
  - Set license to "MPL-2.0"
  - Set version to "0.1.0"
  - Set edition to "2021"
  - Add authors field with project maintainer
  - Verify `cargo metadata` returns expected metadata values

- [x] Create LICENSE file with MPL-2.0 text
  - Download official Mozilla Public License 2.0 text
  - Create LICENSE file in repository root
  - Include copyright notice with current year
  - Verify LICENSE file matches official MPL-2.0 format

- [x] Add MPL-2.0 headers to Rust source files
  - Create MPL-2.0 header template with copyright and license reference
  - Add header comment block to src/main.rs
  - Add header comment block to src/lib.rs
  - Verify headers include required MPL-2.0 notice and copyright

- [x] Add CLI argument parsing with error handling
  - Add lexopt dependency to Cargo.toml
  - Create CLI argument parsing structure in main.rs
  - Write tests for argument parsing success cases
  - Write tests for invalid argument error handling
  - Create CliError type for argument parsing errors
  - Implement Display trait for clear CLI error messages
  - Verify error messages guide users to correct usage

- [x] Add logging infrastructure
  - Write tests for log output at different levels (error, warn, info, debug)
  - Write tests for log format consistency
  - Add env_logger dependency to Cargo.toml
  - Configure default log level and output formatting
  - Create logging utility functions for CLI success/error messages
  - Verify logging works correctly with different verbosity levels

- [ ] Add TOML configuration parsing support
  - Write tests for parsing valid .noemoji.toml configuration
  - Write tests for handling invalid TOML with descriptive error messages
  - Write tests for Config struct deserialization
  - Add serde dependency with derive feature to Cargo.toml
  - Add toml dependency to Cargo.toml
  - Create Config struct with serde Deserialize trait
  - Create ConfigError type for TOML parsing and validation errors
  - Implement From<toml::de::Error> for ConfigError conversion
  - Implement load_config() function for hierarchical file discovery
  - Verify error messages provide actionable guidance for fixing TOML issues

- [x] Implement --help flag with lexopt
  - Write tests for --help flag output format and content
  - Write tests for help text accuracy and completeness
  - Implement --help flag with usage examples and flag descriptions
  - Verify help text displays correctly and is user-friendly

- [x] Implement --version flag with lexopt
  - Write tests for --version flag output format
  - Write tests for version information accuracy
  - Extend CLI argument parsing to handle --version flag
  - Implement --version flag showing version from Cargo.toml
  - Include additional version info (build date, commit hash if available)
  - Verify version display follows standard CLI conventions

- [ ] Document dependency selection rationale
  - Create DEPENDENCIES.md file documenting library choices
  - Document why lexopt was chosen over clap for CLI parsing
  - Document why env_logger was chosen for logging
  - Document why serde+toml was chosen for configuration
  - Include evaluation criteria: binary size, compilation time, features
  - Verify all dependencies are actively maintained and MPL-2.0 compatible

- [ ] Create basic README stub
  - Create README.md with project title "noemoji-rs"
  - Add tagline: "A Rust CLI tool for enforcing ASCII-first typography guidelines"
  - Add placeholder sections for Installation, Usage, Configuration, and License
  - Include brief description of ASCII-first typography enforcement
  - Add MPL-2.0 license badge and reference
  - Verify README renders correctly on GitHub

- [ ] Create CONTRIBUTING.md with licensing requirements
  - Document MPL-2.0 contributor requirements and DCO process
  - Add instructions for proper license header inclusion
  - Specify dependency license compatibility requirements
  - Include code style guidelines (rustfmt, clippy)
  - Add pull request and issue templates guidance
  - Verify documentation is comprehensive for new contributors

- [ ] Configure rustfmt for code formatting
  - Create rustfmt.toml with project style preferences
  - Set tab_spaces=4, max_width=100, edition="2021"
  - Configure import ordering and formatting rules
  - Add rustfmt configuration for consistent code style
  - Verify `cargo fmt --check` runs successfully on existing code

- [ ] Configure clippy for linting
  - Create .clippy.toml with linter rules configuration
  - Enable recommended lints for CLI applications
  - Configure specific lints for Unicode handling code
  - Set lint levels for pedantic and nursery lints
  - Verify `cargo clippy --all-targets` runs without warnings

- [ ] Set up GitHub Actions CI pipeline
  - Create .github/workflows directory structure
  - Create ci.yml workflow for automated testing on push/PR
  - Add cargo test job with multiple Rust versions (stable, beta)
  - Add cargo fmt --check job for code formatting validation
  - Add cargo clippy job for linting with deny warnings
  - Add cargo audit job for security vulnerability scanning
  - Verify CI workflow runs successfully and catches quality issues
