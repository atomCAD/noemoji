# Plan: Unicode Compliance Linter (noemoji-rs)

A Rust-based unicode compliance linter that removes obvious AI authorship signatures - detects and flags emojis and decorative Unicode characters that AI assistants frequently insert, and which detract from professional, clean code appearance.

## Outcomes

### Core Detection Engine

- Detect Unicode characters from defined categories that have ASCII equivalents
  - Success criteria:
    - Identifies emojis (all Unicode emoji ranges), some with ASCII equivalents e.g. :)
    - Detects fancy arrows (→, ←, ⇒, ↑, ↓ etc.) vs ASCII equivalents
    - Finds checkmarks/crosses (✓, ✔, ✗, ✘) vs bracket notation
    - Spots box drawing characters (┌, ─, │, └) vs ASCII art
    - Catches mathematical symbols with ASCII alternatives (≤, ≥, ≠, ±, ×, ÷)
    - Identifies superscripts/subscripts (²³⁴, ₁₂₃) vs caret/underscore notation
    - Finds fraction characters (½, ¾) vs slash notation
    - Detects other symbols with ASCII alternatives (★, ●, ♦)
  - Performance constraint: Process large files efficiently (8-11x faster than grapheme-based approaches)
  - Architecture: Manual character iteration using `.contains()` syntax with static Unicode range arrays
  - Principle: Category-based detection without contextual analysis or grapheme clustering
  - Principle: Exemption categories take precedence over detection categories
  - Principle: Any code point >0x7f is a violation unless exemption applies
  - Implementation: Hybrid enum/static array design for optimal performance and maintainability
  - Recommended architecture pattern (benchmarked 8-11x faster than unicode-segmentation):

    ```rust
    trait CharacterClassifier {
        fn matches(&self, c: char) -> Option<Violation>;
    }

    enum Violation {
        Emoji { chr: char, replacement: Option<&'static str> },
        Arrow { chr: char, replacement: Option<&'static str> },
        // Other variants...
    }

    struct EmojiClassifier;

    impl CharacterClassifier for EmojiClassifier {
        fn matches(&self, c: char) -> Option<Violation> {
            const RANGES: &[(char, char)] = &[
                ('\u{1F600}', '\u{1F64F}'), // Emoticons
                ('\u{1F300}', '\u{1F5FF}'), // Misc symbols & pictographs
                // etc.
            ];

            if RANGES.iter().any(|(start, end)| (*start..=*end).contains(&c)) {
                match c {
                    '\u{2705}' => Some(Violation::Checkmark { chr: c, replacement: Some("[x]") }),
                    _ => Some(Violation::Emoji { chr: c, replacement: None }),
                }
            } else {
                None
            }
        }
    }
    ```

  - Benchmarking results:
    - Manual char iteration: 8-11x faster than unicode-segmentation (364ns vs 2.5-7.1µs on sample text)
    - Large text processing: 8.4x faster (2.7µs vs 23µs on 2KB text)
    - Static array approach: 8% faster than direct inline checks (344ns vs 376ns)
    - `.contains()` syntax: 15% faster than `match` patterns for same ranges (365ns vs 383ns for 7 ranges)

- Provide conservative ASCII replacement suggestions for clear equivalents only
  - Success criteria:
    - Maps directional arrows to ASCII equivalents (→ becomes "->", ← becomes "<-")
    - Suggests bracket notation for specific checkmarks (✓ becomes "[x]", ✗ becomes "[ ]")
    - Provides programming operators for common math symbols (≤ becomes "<=", ≥ becomes ">=", ≠ becomes "!=")
    - Converts specific super/subscripts (² becomes "^2", ³ becomes "^3")
    - Suggests fractional notation for common fractions (½ becomes "1/2", ¼ becomes "1/4")
    - Most violations flagged without suggestions (emojis, flags, complex symbols)
  - Constraint: Suggestions only for clear, unambiguous, widely accepted alternatives
  - Principle: Conservative approach - flag and eliminate rather than preserve through substitution
  - Principle: ASCII-first enforcement, not Unicode preservation

- Respect limited exemptions for essential non-ASCII content
  - Success criteria:
    - Allows latin diacritics for proper spelling (naïve, résumé, café)
    - Permits non-English languages and international content (世界, Москва)
    - Accepts currency symbols for financial data (e.g. £, ¥, €, ₹, ₽, ₩)
    - Allows technical symbols without ASCII equivalents (e.g. °, ∞, µ)
    - Permits legal/formal symbols used in contracts/documentation (©, ®, ™, §, ¶)
    - Allows proper typography (em dash, en dash, smart quotes) for professional writing
    - All other Unicode characters in detection categories trigger violations
  - Principle: Built-in exemptions provide sensible defaults for legitimate Unicode
  - Principle: Professional content with legitimate Unicode needs vs ASCII-substitutable characters
  - Constraint: Math operators with ASCII alternatives remain flagged (±, ×, ÷, ≤, ≥, ≠)
  - Precedence: detection -> exemption suppresses -> deny re-enables -> allow suppresses

- Offer automatic fixing capabilities with safety measures
  - Success criteria:
    - --fix flag applies ASCII replacements automatically
    - Atomic rename operations preserve original as .bak (no in-place modification)
    - --dry-run previews changes without modification
    - --no-backup uses single atomic rename replacement
    - Reports all changes made with before/after context
  - Principle: Original file is never partially written
  - Safety constraint: Two-rename sequence ensures original exists as backup or live file at all times

- Provide diff output for fix preview
  - Success criteria:
    - --diff flag outputs unified diff format showing proposed changes
    - Diff output compatible with patch command for manual application
    - Shows context lines around changes (default 3, configurable)
    - Diff output goes to stdout, usable in pipelines
    - Combines with --fix to show diff of changes made
  - Principle: Users can review exact changes before or after applying fixes
  - Constraint: Diff format follows unified diff standard (GNU diff -u compatible)

### Configuration System

- Support flexible configuration via TOML files
  - Success criteria:
    - Loads .noemoji.toml from current directory, walking up to git root or filesystem root
    - Multiple configs merge with nearest-to-file precedence for conflicting keys
    - Command-line flags override all configuration file settings
    - Supports character category enable/disable (arrows, checkmarks, box-drawing, math-symbols, etc.)
    - Supports allow lists for specific characters or character ranges (e.g., allow = ["→", "U+2190..U+21FF"])
    - Supports deny lists to re-enable detection for exempted characters (e.g., deny = ["°"] flags degree symbol despite built-in exemption)
    - Supports path patterns for file inclusion/exclusion (e.g., include = ["src/**"], exclude = ["vendor/**"])
    - Validates configuration with clear error messages pointing to problematic lines
  - Constraint: Configuration must be discoverable and hierarchical
  - Principle: Sensible defaults with override capability
  - Principle: Configuration discovery follows standard tool behavior (git, ripgrep patterns)

- Support inline comment exemptions for per-line suppression
  - Success criteria:
    - `# noemoji: ignore` suppresses violations on the current line
    - `# noemoji: ignore-next-line` suppresses violations on the following line
    - `# noemoji: ignore-start` / `# noemoji: ignore-end` suppresses violations in a block
    - Exemption comments work in any language with appropriate comment syntax (// for Rust/C/JS, # for Python/Ruby/Shell)
    - Invalid exemption syntax reported as warnings
    - --report-exemptions flag lists all exemptions used in a run
  - Principle: Granular control without global config changes
  - Constraint: Exemption directives must be in comments, not strings or code

- Support baseline files for gradual adoption
  - Success criteria:
    - --generate-baseline creates .noemoji-baseline.json from current violations
    - --baseline flag compares against baseline, only reports new violations
    - Baseline file records file path, line, column, character, and content hash
    - Stale baseline entries (file changed) reported as warnings
    - --update-baseline refreshes baseline with current violations
    - Baseline file format is human-readable and diff-friendly
  - Principle: Enable "no new violations" CI enforcement without fixing legacy issues
  - Constraint: Baseline comparison must be fast (hash-based, not re-scanning)

- Support environment variable configuration
  - Success criteria:
    - NOEMOJI_CONFIG specifies config file path (overrides discovery)
    - NOEMOJI_FORMAT sets default output format
    - NOEMOJI_COLOR controls color output (auto/always/never)
    - All boolean flags have NOEMOJI_* equivalents (e.g., NOEMOJI_FIX=1)
    - Environment variables override config file, CLI flags override environment
  - Principle: CI-friendly configuration without command-line complexity
  - Constraint: Precedence order is config file < environment < CLI flags

- Provide introspection commands for configuration discovery
  - Success criteria:
    - --list-categories shows all detection categories with enabled/disabled status
    - --show-config displays effective merged configuration from all sources
    - --show-config identifies which file set each value (for debugging config issues)
    - Both commands output in human-readable format by default, JSON with --format json
  - Principle: Users can discover and debug configuration without reading documentation

### File Processing

- Handle diverse file processing scenarios with Unix-like output
  - Success criteria:
    - Input modes:
      - Processes individual files specified as arguments
      - Processes glob patterns (e.g., **/*.rs, src/**/*.md)
      - Supports recursive directory traversal
      - Accepts stdin for pipeline integration
    - Filtering behavior:
      - Respects .gitignore files when processing directories (--no-ignore to bypass)
      - Respects .noemojiignore files for tool-specific exclusions (same syntax as .gitignore)
      - Applies extension filtering to discovered files (via config or flags)
      - Binary detection skips non-text files automatically
      - UTF-8 validation skips files with invalid encoding
    - Symlink handling:
      - Symlinks followed by default (matches user intent when symlinks exist in project)
      - --no-follow-symlinks flag disables following symlinks
      - Circular symlink detection prevents infinite traversal
      - Each real path processed only once (deduplication via canonical path)
    - Warning/skipping behavior:
      - Warns to stderr when explicit files are skipped (binary/non-UTF-8/permission errors)
      - Silently skips filtered discovered files (no warnings for filtered directory traversal)
    - Output format:
      - Outputs violations line-by-line as found (file:line:column format)
      - Streams results incrementally (no buffering)
      - Sends processing errors to stderr, violations to stdout
    - Path output:
      - Paths reported relative to current working directory by default
      - --absolute-paths flag outputs absolute paths
      - Paths normalized (no ./ prefix, consistent separators)
  - Performance constraint: Efficiently processes large codebases
  - Principle: Honor user intent for explicit files (warn if skipped)
  - Principle: Standard Unix tool behavior and conventions

- Tool processes codebases efficiently at scale
  - Success criteria:
    - Large files process without excessive memory consumption
    - Large codebases (many files) complete in reasonable time
    - Memory usage scales with chunk size, not file or line size
    - Parallel processing utilizes available CPU cores effectively
    - Binary size remains reasonable for easy distribution
    - Compilation time supports rapid development iteration
    - Performance comparable to or better than sequential grep
  - Performance: Streaming architecture prevents memory buildup
  - Performance: Incremental output enables early result visibility
  - Constraint: Must handle files larger than available RAM
  - Constraint: Chunked reading with fixed-size buffers (not line-based) handles files with arbitrarily long lines
  - Principle: Performance should not compromise correctness

- Cache results to accelerate repeated runs
  - Success criteria:
    - File content hashes stored in .noemoji-cache directory
    - Unchanged files (same hash) skip re-scanning on subsequent runs
    - Cache invalidated when config changes affect detection
    - --no-cache flag bypasses cache for fresh scan
    - --clear-cache removes cached data
    - Cache is machine-local, not committed to version control
  - Performance: 10x+ speedup on large codebases with few changes
  - Constraint: Cache format versioned to handle tool upgrades
  - Principle: Caching must never produce stale/incorrect results

### Output Formatting

- Support core output formats for common use cases
  - Success criteria:
    - TTY-aware automatic mode selection:
      - TTY detected on stdout: verbose mode (human at terminal)
      - No TTY (pipe/redirect): oneline mode (script/pipeline integration)
      - --oneline flag forces single-line output regardless of TTY
      - --verbose flag forces multi-line output regardless of TTY
    - Oneline mode (default for pipes/scripts):
      - One violation per line for grep-compatible output
      - Format: `{path}:{line}:{column}: {category}: {character} [{suggestion}]`
      - Suggestion in brackets when available, omitted when no ASCII equivalent
      - Example: `src/main.rs:42:5: fancy arrow: → [->]`
      - Example without suggestion: `src/main.rs:43:10: emoji: 🚀`
    - Verbose mode (default for TTY) displays rustc-like multi-line helpful errors:
      - Shows source line with caret pointing to violation
      - Includes Unicode analysis (block name, code point, encoding)
      - Displays suggestion with visual diff when available
      - Example:

        ```text
        error: fancy arrow detected
         --> src/main.rs:42:5
           |
        42 | let x → y;
           |       ^ U+2192 'RIGHTWARDS ARROW' (Arrows block)
           |
           = suggestion: use '->' instead
        ```

    - JSON format provides structured data parseable by standard tools (jq, etc.)
    - GitHub Actions format creates ::error:: annotations visible in CI runs
    - Quiet mode produces no output except errors, returns correct exit codes
    - Terminal features (color, width) adapt gracefully in different environments
  - Constraint: Format selection via --format flag with validation
  - Principle: Optimal default for context (human vs script)
  - Principle: Explicit flags override automatic detection

- Support extended output formats for specialized integrations
  - Success criteria:
    - SARIF format enables integration with security and code analysis tools
    - Checkstyle XML format supports build system integration (Jenkins, Maven, Gradle)
    - CSV/TSV formats enable data analysis and reporting workflows
    - All formats provide identical violation information (no data loss across formats)
    - XML output properly escapes special characters preventing injection attacks
    - JSON output uses standard escaping for all string content
    - SARIF format validates against schema preventing malformed output
  - Constraint: Formatters implement common OutputFormatter trait
  - Quality: Format consistency validated through comprehensive test matrix

- Handle output streaming and process integration correctly
  - Success criteria:
    - Output streams correctly route violations to stdout and errors to stderr
    - Exit codes follow standard conventions (0=success, 1=violations, 2=errors)
    - Streaming output handles large violation counts efficiently
    - Memory usage remains constant regardless of violation count
    - Format validation provides clear error messages for invalid options
    - SIGPIPE handled gracefully (exit 0 when downstream closes pipe, e.g., `noemoji file | head -1`)
    - SIGINT terminates processing cleanly without corrupting state
    - Partial output flushed before termination on interrupt
  - Constraint: All output must be streaming-capable for large datasets
  - Performance: Constant memory usage regardless of output size
  - Principle: Standard Unix conventions for streams and exit codes

- Enable output format extensibility and customization
  - Success criteria:
    - Custom format templates enable project-specific output styles
    - Format capability negotiation supports feature detection (color, streaming, etc.)
    - OutputFormatter trait allows plugin-based format additions
    - Format plugins can be developed without modifying core code
  - Principle: Output format extensibility through trait-based plugin architecture
  - Constraint: Plugin API maintains stability across versions

### User Experience

- Command-line interface follows Unix philosophy for intuitive usage
  - Success criteria:
    - Tool accepts files as arguments or stdin for pipeline integration
    - Help text is comprehensive with usage examples
    - Flags are intuitive and follow common CLI conventions
    - Progress indicators for long operations (when outputting to TTY)
    - Command-line arguments parsed according to POSIX conventions
    - Subcommands or flag groups organized logically
  - Principle: Do one thing well, composable with other Unix tools
  - Principle: Prefer simple flags over complex option syntax
  - Constraint: CLI argument parsing uses lexopt for standards compliance

- Error handling guides users toward successful resolution
  - Success criteria:
    - Input validation:
      - Invalid glob patterns report syntax errors with examples
      - Nonexistent files report clear "not found" errors with path
      - Permission errors specify which file and what permission is needed
      - Non-UTF-8 files report encoding issues with file path
    - Configuration errors:
      - Point to exact line number and describe what's wrong
      - Suggest valid alternatives for invalid values
      - Detect conflicting configuration options
    - Command-line errors:
      - Invalid arguments show correct usage with examples
      - Unknown flags suggest similar valid flags
      - Missing required arguments explain what's needed
    - Processing errors:
      - Unicode processing errors include character context and code point details
      - Suggestions provided for common mistakes and misconfigurations
      - Error messages written in plain language, not technical jargon
    - Error recovery:
      - Recoverable errors continue processing other files when appropriate
      - Fatal errors exit immediately with clear explanation
  - Principle: Fail fast with actionable guidance
  - Principle: Error messages should teach, not just report
  - Constraint: Errors route to stderr, not stdout

- Users can easily install and update the tool
  - Success criteria:
    - Published to crates.io for cargo install distribution
    - GitHub releases include pre-built binaries for major platforms (Linux, macOS, Windows)
    - Release binaries are statically linked where possible for portability
    - CI automatically builds and tests releases across platforms
    - Version numbering follows semantic versioning (semver)
    - Release process documented for maintainers
    - Installation instructions cover all distribution methods
  - Principle: Lower barriers to adoption through multiple distribution channels
  - Constraint: Binary releases must be reproducible from tagged source
  - Security: Release artifacts signed or checksummed for integrity verification

- Shell completions enable efficient CLI usage
  - Success criteria:
    - Bash completions generated and included in releases
    - Zsh completions generated and included in releases
    - Fish completions generated and included in releases
    - PowerShell completions generated and included in releases
    - `--generate-completions <shell>` outputs completion script to stdout
    - Completions cover all flags, subcommands, and file arguments
  - Principle: Tab completion is expected for modern CLI tools
  - Constraint: Completions generated from argument parser, not manually maintained

- Man pages provide offline documentation
  - Success criteria:
    - Man page generated in roff format for Unix systems
    - --generate-man outputs man page to stdout
    - Man page included in release archives
    - Man page covers all commands, flags, and configuration options
    - Man page includes examples section
  - Principle: Unix tools should have man pages
  - Constraint: Man page generated from help text, not manually maintained

- Documentation enables users to adopt and use the tool effectively
  - Success criteria:
    - README covers installation methods and quick start guide
    - Usage examples demonstrate common scenarios and workflows
    - Configuration documentation explains all available options
    - CLI help text provides quick reference without leaving terminal
    - Error messages guide users toward resolution
    - Contributing guide enables community participation
    - CHANGELOG documents version history and migration notes
  - Principle: Documentation maintained alongside code changes
  - Principle: Examples over exhaustive reference material
  - Constraint: Documentation accuracy verified through automated testing where possible

### Developer Workflow Integration

- Git integration enables efficient pre-commit workflows
  - Success criteria:
    - --staged flag checks only git staged files
    - `--git-diff <ref>` checks only files changed since ref (branch, commit, tag)
    - --git-diff (no arg) checks files changed since merge-base with default branch
    - Handles renamed/moved files correctly
    - Works with partial staging (staged hunks vs unstaged hunks in same file)
  - Principle: Check only what's being committed, not entire repository
  - Constraint: Requires git repository, graceful error otherwise

- Watch mode enables continuous development feedback
  - Success criteria:
    - --watch flag monitors specified paths for changes
    - Re-runs check on file save with debouncing (100ms default)
    - Clears terminal and shows fresh results on each run
    - Displays timestamp and changed file that triggered re-run
    - Ctrl+C exits cleanly
    - `--watch-debounce <ms>` configures debounce interval
  - Principle: Immediate feedback during development
  - Constraint: Uses notify crate for cross-platform file watching

- Pre-commit framework integration enables standardized git hooks
  - Success criteria:
    - .pre-commit-hooks.yaml in repository root defines noemoji hook
    - Hook works with pre-commit.com framework out of the box
    - Supports language: rust for source installation
    - Supports language: system for binary installation
    - Hook respects --staged by default for pre-commit context
    - Documentation covers pre-commit setup
  - Principle: Integrate with existing ecosystem, don't reinvent
  - Constraint: Hook definition follows pre-commit.com specifications

### Quality Assurance

- Comprehensive test coverage validates tool reliability and correctness
  - Success criteria:
    - Unit tests cover all character detection rules and edge cases
    - Integration tests verify file processing workflows and output formats
    - Property-based tests validate Unicode handling across character ranges
    - Regression tests prevent reintroduction of fixed issues
    - Test suite runs in CI on multiple platforms (Linux, macOS, Windows)
    - Tests validate both positive cases (violations found) and negative cases (clean files)
    - Performance regression tests detect slowdowns in processing
  - Principle: Test-driven development guides implementation
  - Principle: Tests serve as living documentation of expected behavior
  - Constraint: Test suite completes quickly enough for rapid development iteration

- Validation infrastructure ensures code quality through check.sh script
  - Success criteria:
    - check.sh runs all relevant quality checks (tests, lints, formatting)
    - Script exits with non-zero status when violations found
    - Script provides clear output identifying specific problems
    - Runs efficiently enough for pre-commit workflow usage
    - Integrates with CI for automated validation
    - Documents all checks performed and their purpose
  - Principle: Automated validation prevents quality regressions
  - Principle: Same checks run locally and in CI (consistency)
  - Constraint: check.sh must be the single source of truth for validation

- Tool validates its own codebase through dogfooding integration
  - Success criteria:
    - Tool successfully processes its own source code without false positives
    - Configuration tuned to project's legitimate Unicode usage patterns
    - Violations in project codebase fixed or explicitly allowed via config
    - noemoji integrated into check.sh for automated validation
    - Dogfooding reveals usability issues and guides improvements
    - README documents dogfooding as validation approach
  - Principle: Dogfooding builds confidence and validates real-world usability
  - Principle: Self-hosting demonstrates tool capabilities credibly
  - Constraint: Tool must handle its own codebase cleanly (no ignored failures)

### Observability

- Debug logging enables troubleshooting unexpected behavior
  - Success criteria:
    - NOEMOJI_LOG=debug enables verbose internal state logging
    - Configuration source tracing shows which file set which value
    - Detection decision tracing explains why character was/wasn't flagged
    - Logs include file path, line number, and character context for each decision
  - Principle: Users can diagnose unexpected behavior without reading source code
  - Constraint: Debug output goes to stderr, never interferes with stdout

### Project Infrastructure

- Make optimal dependency choices to meet project requirements
  - Success criteria:
    - CLI parsing uses lexopt for lightweight, standards-compliant argument handling
    - Unicode processing uses minimal, focused libraries for specific tasks
    - Configuration uses serde + toml for robust TOML parsing with validation
    - File processing uses walkdir + ignore for efficient directory traversal
    - Output formatting uses appropriate libraries for each target format
    - Dependencies are minimal, well-maintained, and have clear licensing
  - Principle: Prefer smaller, focused libraries over large frameworks
  - Constraint: All dependencies must be actively maintained with recent updates
  - Performance: Minimize compilation time and binary size through careful selection

- Supply chain security integrated into CI
  - Success criteria:
    - cargo-audit runs on every PR and blocks merge on known vulnerabilities
    - Dependency licenses verified compatible with MPL-2.0 (cargo-deny or equivalent)
    - Dependabot or similar enables automated security updates
    - RUSTSEC advisory database checked in CI pipeline
  - Principle: Trust but verify dependencies
  - Constraint: Security checks must not significantly slow CI iteration

- Establish open source project infrastructure
  - Success criteria:
    - Licensing:
      - Project licensed under Mozilla Public License 2.0 (MPL-2.0)
      - All source code files include appropriate MPL-2.0 headers
      - LICENSE file contains full MPL-2.0 license text
      - Dependency licenses compatible with MPL-2.0 (no GPL conflicts)
    - GitHub CI/CD integration:
      - Automated testing runs on all PRs and commits
      - Multi-platform builds validate Linux, macOS, Windows
      - Linting and formatting checks enforce code quality
      - Security scanning detects vulnerabilities
      - Release automation builds and publishes artifacts
    - Project documentation:
      - README documents project purpose, installation, and usage
      - Release process documented for maintainers
    - Community infrastructure:
      - GitHub Discussions or issue labels for questions
      - Clear maintainer response expectations
      - Contribution recognition and attribution process
  - Principle: MPL-2.0 is the only copyleft license compatible with Rust's static linking model
  - Principle: Automated quality gates maintain code quality without manual oversight
  - Principle: Clear processes enable community participation
  - Rationale: LGPL becomes effectively GPL with static linking; MPL-2.0's file-level copyleft works correctly
  - Constraint: All dependencies must be compatible with MPL-2.0 requirements
  - Constraint: CI workflows must be efficient enough for rapid iteration

## Milestones

- MVP achieves minimal end-to-end operational completeness enabling full end-to-end testing
  - Success criteria:
    - Minimal project documentation:
      - LICENSE file present (MPL-2.0)
      - README covers installation, usage, and configuration
    - Minimal quality gates:
      - check.sh script runs complete validation suite:
        - cargo test (unit and integration tests)
        - cargo clippy (Rust linting)
        - cargo fmt --check (formatting)
        - cargo doc --no-deps (documentation builds)
        - markdownlint (markdown quality)
        - shellcheck (shell script quality)
        - Exit code 1 if any check fails, 0 if all pass
      - Core detection logic has unit test coverage
      - At least one integration test validates end-to-end flow
      - No critical bugs that prevent basic operation
    - Supply chain security in CI:
      - cargo-audit runs on every PR and blocks merge on known vulnerabilities
      - Dependency licenses verified compatible with MPL-2.0 (cargo-deny or equivalent)
      - Dependabot enabled for automated security updates
      - RUSTSEC advisory database checked in CI pipeline
    - Multi-platform CI validation:
      - GitHub Actions matrix validates cargo test on Linux, macOS, Windows
      - Ensures code portability even though only Linux binary released in MVP
      - CI catches platform-specific issues early
      - Single Linux x64 pre-built binary sufficient for MVP dogfooding
    - Dependency choice documentation:
      - Document each dependency's justification (minimal, well-maintained, clear licensing)
      - Verify dependencies are actively maintained with recent updates
      - Track binary size contribution for reasonable distribution
      - Serves as foundation for future dependency decisions
    - Core detection pipeline operational:
      - Detects at least one category of violations (e.g. emoji)
      - Provides ASCII replacement suggestions for detected violations (e.g. → to ->)
      - Respects at least one exemption category (e.g. international text)
    - Minimal file handling:
      - Processes UTF-8 text files
      - Skips non-UTF-8 files as binary files with warning
      - Handles basic error cases (file not found, permission denied)
    - Performance constraints (MVP minimal bar):
      - Processes files without loading entire content into memory
      - Handles files larger than available RAM without failure
      - Performance should not be noticeably slower than sequential grep for similar workloads
      - Binary size under 10MB for reasonable distribution
    - Minimal input/output functional:
      - Input modes:
        - Accepts file paths as arguments (single or multiple files)
        - Accepts stdin when no arguments provided or `-` specified
        - Detects TTY on stdin with no file arguments and shows usage help (avoid hanging)
      - Output behavior:
        - Outputs violations line-by-line as found (streaming, not buffered)
        - Stdout flushed after each violation for real-time visibility
        - Uses file:line:column format for violations
        - Sends violations to stdout, errors/warnings to stderr
        - Reports paths relative to current working directory
        - SIGPIPE handled gracefully (exit 0 when downstream closes pipe, e.g., `noemoji file | head -1`)
      - Exit codes work correctly (0=clean, 1=violations, 2=errors)
      - Output formats: oneline mode only (single-line per violation)
        - Format: `{path}:{line}:{column}: {category}: {character} [{suggestion}]`
        - Suggestion in brackets when available, omitted when no ASCII equivalent
        - Example: `src/main.rs:42:5: fancy arrow: → [->]`
        - Example without suggestion: `src/main.rs:43:10: emoji: 🚀`
        - Multiple violations on same line reported separately, ordered by column
        - Stdin path representation: `stdin` as path when processing stdin
        - Quiet mode support (--quiet/-q flag):
          - Violations detected but not displayed (silent operation)
          - Exit code 1 if violations found, 0 if clean, 2 on errors
          - Only outputs processing errors to stderr
          - Performance: Full Unicode scanning still performed
    - Help/usage display:
      - --help/-h flag displays usage information
      - Help text shows basic usage pattern: noemoji [OPTIONS] [FILES...]
      - Lists core flags: --quiet/-q, --staged, --help/-h
      - Help text uses clear language suitable for first-time users
    - Basic configuration support:
      - Loads .noemoji.toml from current working directory
      - CLI flags override config file settings
    - Basic diagnostic support:
      - NOEMOJI_LOG=debug enables diagnostic output to stderr
      - Diagnostic output includes:
        - Configuration discovery: which .noemoji.toml files loaded
        - Final merged configuration values shown at startup
        - Per-violation detection explanation: character, code point, category, suggestion
        - Exemption application when violations suppressed
      - Debug output never interferes with stdout (violations/results)
    - Git integration for pre-commit workflows:
      - --staged flag checks only git staged files
      - Errors gracefully with clear message if not in git repository
      - Works with partial staging (staged hunks vs unstaged hunks in same file)
    - Pre-commit framework integration:
      - .pre-commit-hooks.yaml in repository root defines noemoji hook
      - Hook uses --staged flag by default
      - Language: system for binary installation
      - Documentation in README shows pre-commit setup example
    - End-to-end operational:
      - Tool runs successfully against real codebases (including its own)
      - Complete flow works: input -> detection -> output -> exit code
      - Integration tests can be written against the working pipeline
      - Enables transition to full TDD workflow with comprehensive test coverage
    - Dogfooding: Complete integration with automated enforcement
      - Tool successfully processes its own source code without false positives
      - Configuration tuned to project's legitimate Unicode usage patterns (.noemoji.toml in root)
      - noemoji integrated into check.sh as automated validation step
      - check.sh fails if noemoji detects violations in project codebase
    - Installation options:
      - README.md includes prerequisites (Rust toolchain)
      - cargo install --path . command documented for local installation
      - cargo build instructions for development
      - One pre-built binary for common platform (Linux x64) to enable testing by non-Rust users
      - Manual GitHub release with single binary
    - README.md minimal content:
      - Project description (what noemoji detects and why)
      - Installation from source (cargo commands)
      - Basic usage example (noemoji file.txt or cat file.txt | noemoji)
      - Usage patterns: file arguments and stdin pipeline
      - Exit code meanings (0=clean, 1=violations found, 2=error)
      - Basic .noemoji.toml example (one category toggle)
      - Developer workflow section:
        - Pre-commit.com installation example (.pre-commit-config.yaml)
        - Manual git hook example (scripts/pre-commit sample)
        - Exit code behavior in pre-commit context (1 blocks commit)
      - Reference to --help for complete options
    - Minimal release:
      - Version 0.1.0 tagged in git
      - LICENSE file present (MPL-2.0)
    - Error message clarity:
      - Errors include file path and specific problem
      - File errors use Unix-style format: "{path}: {error}" (e.g., "file.txt: No such file or directory")
      - Unknown flags: "unknown option '{flag}'"
      - CLI errors suggest help: "Try '{program} --help' for more information."
      - Errors routed to stderr, not stdout
  - Principle: Sprint to minimal end-to-end completeness, not feature completeness
  - Principle: MVP enables TDD by providing working integration test target
  - Principle: Early user release provides real-world feedback for prioritization
  - Principle: Minimal viable means "barely sufficient to be useful", not comfortable
  - Constraint: MVP quality sufficient for dogfooding and early adopters, not general release

## Tasks

### Project Infrastructure Tasks

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
  - Set description to "Unicode compliance linter that removes obvious AI authorship signatures"
  - Set license to "MPL-2.0"
  - Set version to "0.1.0"
  - Set edition to "2024"
  - Add authors field with project maintainer
  - Verify `cargo metadata` returns expected metadata values

- [x] Create LICENSE file with MPL-2.0 text
  - Download official Mozilla Public License 2.0 text
  - Create LICENSE file in repository root
  - Include copyright notice with current year
  - Verify LICENSE file matches official MPL-2.0 format

- [x] Add MPL-2.0 headers to Rust source files
  - Create MPL-2.0 header template with copyright and license reference
  - Add header comment block to all src/*.rs files
  - Verify headers include required MPL-2.0 notice and license reference

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

- [x] Add LogLevel enum and wire into init_logger
  - Create LogLevel enum (Disabled, Error, Warn, Info, Debug, Trace) in src/logging.rs
  - Add LogLevel::to_level_filter() for conversion to log::LevelFilter
  - Update init_logger to accept LogLevel parameter
  - Write tests for LogLevel::to_level_filter() conversion

- [x] Add Config and LogConfig structs with pub fields
  - Create LogConfig struct with `pub level: Option<LogLevel>` in src/config.rs
  - Create Config struct with `pub log: LogConfig` in src/config.rs
  - Implement Default trait for both (level: None = inherit/use default)
  - Write tests for Config::default() and field access
  - Update main.rs to use Config with `config.log.level.unwrap_or_default()`

- [x] Add NOEMOJI_LOG environment variable support
  - Use env_logger's filter_or() to check NOEMOJI_LOG with fallback to RUST_LOG
  - Support full env_logger filter syntax (module-level filtering)
  - Write tests for NOEMOJI_LOG taking precedence over RUST_LOG
  - Write tests for RUST_LOG fallback when NOEMOJI_LOG unset
  - Default to off (silent) when neither env var is set

- [x] Add TOML parsing with ConfigError type
  - Write tests for parsing valid .noemoji.toml configuration
  - Write tests for Config struct field deserialization ([log] section with level)
  - Write tests for ConfigError from invalid TOML (syntax errors, type mismatches)
  - Write tests for ConfigError Display implementation with actionable messages
  - Add serde dependency with derive feature to Cargo.toml
  - Add toml dependency to Cargo.toml
  - Add serde Deserialize trait to Config struct
  - Create ConfigError type for TOML parsing and validation errors
  - Implement Display trait for ConfigError with user-friendly error formatting
  - Implement From<toml::de::Error> for ConfigError conversion
  - Implement parse_config() function to parse TOML string into Config with error handling
  - Verify Config correctly deserializes all supported fields
  - Verify error messages provide actionable guidance for fixing TOML issues

- [x] Implement hierarchical configuration file discovery
  - Write tests for finding .noemoji.toml in current directory
  - Write tests for searching parent directories up to filesystem root
  - Write tests for handling missing configuration files (use Config::default())
  - Write tests for handling file read errors (permissions, I/O errors)
  - Implement load_config() function that discovers and loads configuration files
  - Integrate file discovery with existing TOML parsing logic
  - Verify hierarchical search works correctly across directory structures

- [x] Implement configuration merging with inheritance control
  - Write tests for Config::or() with Option field precedence
  - Write tests for Config::load() finding multiple configs
  - Write tests for inherit = false stopping directory traversal
  - Write tests for Config::default() as base when no configs found
  - Write tests for partial configurations overriding base values
  - Add inherit: bool field to Config struct with Default::default() = true
  - Implement Config::or(self, other: Self) -> Self consuming method
  - Implement Config::load(start_dir) that discovers and merges configs
  - Verify merging applies configs from general to specific (child overrides parent)
  - Verify inherit = false in any config stops further parent directory scanning

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
  - Verify version display follows standard CLI conventions

- [x] Add stdin support for pipeline integration
  - Write tests for reading from stdin when no file arguments provided
  - Write tests for explicit `-` argument to read from stdin
  - Write tests for mixing stdin with files (file1.txt - file2.txt)
  - Write tests for processing stdin input with violations
  - Write tests for handling stdin with empty input
  - Write tests for error handling with invalid UTF-8 from stdin
  - Extend CLI argument parsing to accept `-` as stdin placeholder
  - Implement stdin reading in main.rs using io::stdin()
  - Support both patterns: no args defaults to stdin, `-` explicitly requests stdin
  - Verify stdin integration works with Unix pipelines (echo, cat, etc.)
  - Update help text to document both stdin usage patterns

- [x] Implement Outcome enum with Termination trait
  - Write tests for Outcome::Success returning 0
  - Write tests for Outcome::Violations returning 1
  - Write tests for Outcome::Error returning 2
  - Define Outcome enum in cli.rs with Success, Violations, Error variants
  - Implement Termination trait for Outcome to enable returning from main
  - Update main.rs to return Outcome instead of ()
  - Verify exit codes work correctly with shell scripts ($?)
  - Document exit codes in rustdoc comments

- [x] Create basic README stub
  - Create README.md with project title "noemoji-rs"
  - Add tagline from Cargo.toml description field
  - Add placeholder sections for Installation, Usage, Configuration, and License
  - Include brief description expanding on Cargo.toml description
  - Add MPL-2.0 license badge and reference
  - Verify README renders correctly on GitHub

- [x] Create GitHub Actions workflow for CI validation
  - Create .github/workflows/ directory structure
  - Write tests for workflow file YAML syntax validation
  - Create ci.yml workflow file for automated validation on push/PR events
  - Configure workflow to execute ./check.sh script for all quality checks
  - Include Rust toolchain setup (latest stable) with component installation
  - Add build caching for dependencies to improve CI performance
  - Configure test matrix for multi-platform testing (ubuntu-latest, macos-latest, windows-latest)
  - Verify workflow YAML parses correctly and triggers on expected events

- [x] Test GitHub Actions CI pipeline with actual changes
  - Write tests for CI pipeline behavior (success/failure scenarios)
  - Create test commit that should pass all checks
  - Create test commit that should fail quality checks (temporarily)
  - Verify CI runs automatically on push to main branch
  - Verify CI runs on pull request creation and updates
  - Verify CI reports success/failure status correctly
  - Verify CI build artifacts and logs are accessible
  - Confirm CI environment can install required tools (markdownlint-cli2, shellcheck)

- [x] Create check.sh validation script
  - Create check.sh script in repository root
  - Add cargo test execution with proper error handling
  - Add cargo clippy execution with --deny warnings
  - Add cargo fmt --check for formatting verification
  - Add cargo doc --no-deps for documentation validation
  - Add markdownlint-cli2 execution for Markdown files
  - Add shellcheck execution for shell script validation
  - Verify script exits with non-zero status on any check failure
  - Test script locally and in CI environment

- [ ] Create deny.toml with license allowlist for MPL-2.0 compatibility
  - Create deny.toml configuration file in repository root
  - Configure license allowlist for MPL-2.0 compatible licenses
  - Verify all current dependencies pass license check
  - Test that incompatible licenses are rejected

- [ ] Add security advisory checking to deny.toml
  - Add advisories section to deny.toml
  - Configure RUSTSEC advisory database checking
  - Verify known vulnerabilities are detected
  - Test advisory checking catches security issues

- [ ] Integrate cargo-deny into check.sh script
  - Add cargo-deny check execution to check.sh
  - Verify check.sh exits with non-zero on license violations
  - Verify check.sh exits with non-zero on security advisories
  - Test integration with CI workflow

- [ ] Add cargo-audit to check.sh validation script
  - Add cargo-audit execution to check.sh
  - Verify check.sh exits with non-zero on security advisory findings
  - Test that RUSTSEC advisory database is consulted

- [ ] Add cargo-audit to CI workflow
  - Add cargo-audit installation to .github/workflows/ci.yml
  - Add cargo audit check step after cargo test
  - Configure audit to fail CI on known vulnerabilities (exit 1)
  - Add advisory-db cache for faster audit runs
  - Verify audit runs correctly in CI

- [ ] Configure Dependabot for automated security updates
  - Create .github/dependabot.yml configuration file
  - Configure automated updates for Cargo dependencies
  - Set update frequency and reviewer requirements
  - Verify Dependabot creates pull requests for dependency updates

- [ ] Document dependency choices and maintenance status
  - Create DEPENDENCIES.md file documenting each dependency justification
  - Document maintenance status verification process
  - Track binary size contribution of each dependency
  - Verify all dependencies are actively maintained with recent releases
  - Document license compatibility verification results

- [ ] Implement SIGPIPE signal handling
  - Add signal handler for SIGPIPE to exit gracefully with code 0
  - Test pipeline behavior: noemoji file | head -1 exits cleanly
  - Verify handler works across platforms (Unix systems)
  - Test signal handling doesn't interfere with normal operation

- [x] Update file error messages to Unix-style format
  - Update CheckError Display to use "{path}: {error}" format
  - Ensure file errors route to stderr
  - Test file error message formatting

- [x] Update CLI error messages to Unix-style format
  - Update CliError Display to use "unknown option '{flag}'" format
  - Add help suggestion: "Try '{program} --help'"
  - Ensure CLI errors route to stderr
  - Test CLI error message formatting

- [ ] Create Linux x64 release build process
  - Configure release build for x86_64-unknown-linux-gnu target
  - Strip binary symbols for size reduction
  - Verify binary size under 10MB constraint
  - Test binary runs on clean Linux system without Rust toolchain

- [ ] Create scripts/pre-commit example hook script
  - Depends on: Filter violations to only report staged content
  - Create scripts/ directory in repository root
  - Write scripts/pre-commit example hook that runs noemoji --staged
  - Make script executable and include usage instructions in comments
  - Test hook prevents commits with violations

### Core Detection Engine Tasks

#### Infrastructure (Must Complete First)

- [ ] Define CharacterClassifier trait and Violation enum
  - Define CharacterClassifier trait with `matches(&self, c: char) -> Option<Violation>`
  - Define Violation enum with Arrow variant (`char` field, `Option<&'static str>` replacement)
  - Write tests for trait contract and Violation::Arrow

- [ ] Refactor InputSource::check() callback to use Violation
  - Refactor callback from `FnMut(usize, usize, char)` to `FnMut(usize, usize, &Violation)`
  - Update existing call sites in main.rs
  - Write tests for refactored callback integration

#### Arrow Detection

- [ ] Implement ArrowClassifier with full Unicode Arrows block
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Write tests for Arrows block range U+2190-U+21FF (← → ↑ ↓ ⇒ ⇐ ↔ ⇔ etc.)
  - Define ArrowClassifier with const RANGES for Arrows block
  - Map specific characters to replacements (→ to "->", ← to "<-", etc.)

#### Emoji Detection

- [ ] Implement EmojiClassifier with Emoticons range (U+1F600-U+1F64F)
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::Emoji variant (no replacement)
  - Write tests for Emoticons range (😀 😃 😄 😁 😆 😅 etc.)
  - Create EmojiClassifier with const EMOTICONS_RANGE

- [ ] Add Miscellaneous Symbols and Pictographs range to EmojiClassifier
  - Write tests for U+1F300-U+1F5FF range
  - Add MISC_SYMBOLS_PICTOGRAPHS_RANGE const

- [ ] Add Transport and Map Symbols range to EmojiClassifier
  - Write tests for U+1F680-U+1F6FF range
  - Add TRANSPORT_MAP_SYMBOLS_RANGE const

- [ ] Add Supplemental Symbols and Pictographs range to EmojiClassifier
  - Write tests for U+1F900-U+1F9FF range
  - Add SUPPLEMENTAL_SYMBOLS_RANGE const

- [ ] Add Extended-A and Dingbats ranges to EmojiClassifier
  - Write tests for Extended-A range U+1FA00-U+1FA6F
  - Write tests for Dingbats emoji subset U+2700-U+27BF
  - Add EXTENDED_A_RANGE and DINGBATS_RANGE consts

- [ ] Add Miscellaneous Symbols range to EmojiClassifier
  - Write tests for U+2600-U+26FF range
  - Add MISC_SYMBOLS_RANGE const

#### Checkmark Detection

- [ ] Implement CheckmarkClassifier with Unicode ranges
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::Checkmark variant
  - Write tests for checkmark range U+2713-U+2714, U+2705
  - Write tests for cross range U+2717-U+2718
  - Create CheckmarkClassifier with const RANGES
  - Map checkmarks to "[x]", crosses to "[ ]"

#### Box Drawing Detection

- [ ] Implement BoxDrawingClassifier with Unicode block range
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::BoxDrawing variant (no replacement)
  - Write tests for Box Drawing block range U+2500-U+257F
  - Write tests for Block Elements range U+2580-U+259F
  - Create BoxDrawingClassifier with const RANGES

#### Math Symbol Detection

- [ ] Implement MathOperatorClassifier with Unicode ranges
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::MathSymbol variant
  - Write tests for Mathematical Operators block U+2200-U+22FF
  - Write tests for Supplemental Mathematical Operators U+2A00-U+2AFF
  - Create MathOperatorClassifier with const RANGES
  - Map operators to ASCII: ≤ to <=, ≥ to >=, ≠ to !=, × to *, ÷ to /

- [ ] Implement SuperscriptClassifier with Unicode ranges
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::Superscript variant
  - Write tests for superscript range U+2070-U+207F and U+00B2-U+00B3
  - Create SuperscriptClassifier with const RANGES
  - Map to caret notation (^0 through ^9)

- [ ] Implement SubscriptClassifier with Unicode range
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::Subscript variant
  - Write tests for subscript range U+2080-U+208E
  - Create SubscriptClassifier with const RANGES
  - Map to underscore notation (`_0` through `_9`)

- [ ] Implement FractionClassifier with Unicode ranges
  - Depends on: Define CharacterClassifier trait and Violation enum
  - Add Violation::Fraction variant
  - Write tests for Number Forms U+2150-U+215F and Latin-1 fractions U+00BC-U+00BE
  - Create FractionClassifier with const RANGES
  - Map to slash notation (1/4, 1/2, 3/4, etc.)

#### Exemption Categories

- [ ] Implement Latin diacritics exemption via Unicode range
  - Write tests for Latin-1 Supplement range U+0080-U+00FF (accents, umlauts, etc.) not flagged
  - Write tests for Latin Extended-A range U+0100-U+017F not flagged
  - Write tests for Latin Extended-B range U+0180-U+024F not flagged
  - Write tests verifying exempted characters in words (naïve, résumé, café)
  - Create LatinDiacriticsExemption with const RANGES for Latin extended blocks
  - Integrate exemption check in detection flow (return None for exempt chars)
  - Verify range-based diacritics exemption tests pass

- [ ] Implement CJK Unified Ideographs exemption (U+4E00-U+9FFF)
  - Write tests for CJK Unified Ideographs range U+4E00-U+9FFF not flagged
  - Write tests verifying exempted characters in words (世界, 中国, 日本)
  - Create CJKExemption with const RANGES for CJK blocks
  - Integrate CJK exemption into detection flow
  - Verify CJK exemption tests pass

- [ ] Implement Cyrillic script exemption (U+0400-U+04FF)
  - Write tests for Cyrillic range U+0400-U+04FF not flagged
  - Write tests verifying exempted characters in words (Москва, Россия)
  - Create InternationalTextExemption with CYRILLIC_RANGE const
  - Integrate InternationalTextExemption into detection flow
  - Verify Cyrillic exemption tests pass

- [ ] Implement Arabic script exemption (U+0600-U+06FF)
  - Write tests for Arabic range U+0600-U+06FF not flagged
  - Write tests verifying exempted characters in Arabic text
  - Add ARABIC_RANGE to InternationalTextExemption const RANGES
  - Verify Arabic exemption tests pass

- [ ] Implement Hangul Syllables exemption
  - Write tests for Hangul Syllables range U+AC00-U+D7AF not flagged
  - Write tests verifying Korean text (한국어) not flagged
  - Add HANGUL_SYLLABLES_RANGE to InternationalTextExemption

- [ ] Implement Greek and Coptic exemption
  - Write tests for Greek and Coptic range U+0370-U+03FF not flagged
  - Write tests verifying Greek text (Ελληνικά) not flagged
  - Add GREEK_COPTIC_RANGE to InternationalTextExemption

- [ ] Implement Hebrew exemption
  - Write tests for Hebrew range U+0590-U+05FF not flagged
  - Write tests verifying Hebrew text (עברית) not flagged
  - Add HEBREW_RANGE to InternationalTextExemption

- [ ] Implement Devanagari exemption
  - Write tests for Devanagari range U+0900-U+097F not flagged
  - Write tests verifying Hindi text (हिन्दी) not flagged
  - Add DEVANAGARI_RANGE to InternationalTextExemption

- [ ] Implement currency symbol exemption via Unicode range
  - Write tests for Currency Symbols block U+20A0-U+20CF not flagged
  - Write tests for common currency in Latin-1 Supplement (£ U+00A3, ¥ U+00A5) not flagged
  - Write tests for Euro sign U+20AC not flagged
  - Create CurrencyExemption with const RANGES for currency Unicode blocks
  - Integrate currency exemption into detection flow
  - Verify range-based currency exemption tests pass

- [ ] Implement technical symbol exemption via Unicode ranges
  - Write tests for degree symbol (° U+00B0) not flagged
  - Write tests for micro symbol (µ U+00B5) not flagged
  - Write tests for infinity (∞ U+221E) not flagged - specific char in Math Operators
  - Write tests for other technical symbols without ASCII equivalents
  - Create TechnicalSymbolExemption with const RANGES for technical character ranges
  - Note: Some technical symbols are specific characters, not contiguous ranges
  - Integrate technical symbol exemption into detection flow
  - Verify range-based technical symbol exemption tests pass

- [ ] Implement legal symbol exemption via Unicode range
  - Write tests for Letterlike Symbols subset U+2100-U+214F (™ U+2122, etc.) not flagged
  - Write tests for Latin-1 Supplement legal chars (© U+00A9, ® U+00AE) not flagged
  - Write tests for section sign (§ U+00A7), paragraph sign (¶ U+00B6) not flagged
  - Create LegalSymbolExemption with const RANGES for legal symbol characters
  - Integrate legal symbol exemption into detection flow
  - Verify range-based legal symbol exemption tests pass

- [ ] Implement typography exemption via Unicode range
  - Write tests for General Punctuation range U+2000-U+206F not flagged (em dash, en dash, etc.)
  - Write tests for smart quotes (U+2018-U+201F) not flagged
  - Write tests for ellipsis (U+2026) not flagged
  - Create TypographyExemption with const RANGES for typographic characters
  - Integrate typography exemption into detection flow
  - Verify range-based typography exemption tests pass

#### Integration

- [ ] Create detection pipeline scaffold with classifier registration
  - Depends on: Refactor InputSource::check() callback to use Violation
  - Write tests for pipeline with single classifier
  - Write tests for pipeline returning first matching violation
  - Implement pipeline that iterates classifiers in order
  - Verify single-classifier pipeline tests pass

- [ ] Add exemption checking to detection pipeline
  - Depends on: Implement Latin diacritics exemption
  - Write tests for exemptions suppressing violations
  - Write tests for exemptions checked before classifiers
  - Integrate exemption checks into pipeline
  - Verify exemption integration tests pass

- [ ] Refactor character-by-character detection to use classifiers
  - Depends on: Create detection pipeline scaffold, Implement ArrowClassifier
  - Write tests for calling classifiers on each character
  - Write tests for exemptions suppressing violations
  - Refactor check_reader() to use CharacterClassifier trait instead of PROHIBITED_CHARS.contains()
  - Apply exemption and precedence logic
  - Verify detection produces correct violations with positions

- [ ] Wire classifier pipeline into main detection flow
  - Write tests for detection flow processing characters through classifiers
  - Wire classifier pipeline into InputSource::check()
  - Verify detection flow works end-to-end with classifiers

- [ ] Implement category precedence (detection -> exemption -> deny -> allow)
  - Depends on: Integrate allow/deny lists into detection pipeline
  - Write tests for exempted character re-enabled via deny list
  - Write tests for detected character suppressed via allow list
  - Write tests for exempted character remaining exempt (no deny)
  - Write tests for detected character remaining detected (no allow)
  - Add precedence logic: check detection, apply exemptions, apply deny overrides, apply allow overrides
  - Verify precedence test cases pass

- [ ] Verify MVP Core Detection Engine operational end-to-end
  - Depends on: Update violation output, Implement ArrowClassifier, Implement EmojiClassifier with Emoticons range
  - Write integration test: file with emoji produces {path}:{line}:{col}: emoji: {char}
  - Write integration test: file with arrow produces output with [->] suggestion
  - Write integration test: file with café not flagged (Latin diacritics exemption)
  - Write integration test: multiple violations in file reported in order
  - Verify complete flow: file input -> detection -> exemption -> formatted output -> exit code

- [ ] Remove legacy PROHIBITED_CHARS array
  - Depends on: Verify MVP Core Detection Engine operational end-to-end
  - Delete PROHIBITED_CHARS const from check.rs
  - Verify all tests pass without legacy array
  - Confirm ArrowClassifier handles all previously detected arrows

### Configuration System Integration

- [ ] Pass loaded configuration to detection pipeline
  - Depends on: Wire classifier pipeline into main detection flow
  - Write tests for detection pipeline receiving configuration
  - Refactor InputSource::check() to accept Config parameter
  - Update main.rs:16 loaded config to pass through detection (currently discarded after logging)
  - Replace hardcoded PROHIBITED_CHARS array with configurable classifier pipeline
  - Verify configuration integration works correctly

- [ ] Add detection category toggles with integration
  - Write tests for category toggles affecting classifier selection
  - Write tests for [detection] section with arrows = false disabling arrow detection
  - Write tests for [detection] section with emojis = false disabling emoji detection
  - Write tests for multiple category toggles in same config
  - Write tests for default behavior (all enabled) when categories not specified
  - Write tests for category toggles affecting detection output
  - Create DetectionConfig struct with `Option<bool>` fields: arrows, emojis, checkmarks, box_drawing, math_symbols, superscripts, subscripts, fractions
  - Add `detection: Option<DetectionConfig>` field to Config
  - Integrate category toggles into detection pipeline (skip disabled classifiers)
  - Verify category toggle integration tests pass

- [ ] Add allow and deny fields to Config struct
  - Write tests for allow/deny configuration parsing from TOML
  - Add `allow: Option<Vec<String>>` and `deny: Option<Vec<String>>` to Config struct
  - Remove Copy trait from Config (Vec fields are not Copy)
  - Update Config::or() merge logic to handle Vec fields
  - Verify config parsing tests pass

- [ ] Implement character specification parsing for allow/deny lists
  - Write tests for single character syntax ("→", "°")
  - Write tests for Unicode range syntax ("U+2190..U+21FF")
  - Write tests for invalid syntax producing clear error messages
  - Implement parse_char_spec() for parsing character and range syntax
  - Verify parsing tests pass

- [ ] Integrate allow/deny lists into detection pipeline
  - Write tests for allow list suppressing specific detected characters
  - Write tests for deny list re-enabling exempted characters
  - Write tests for precedence: detection -> exemption -> deny -> allow
  - Integrate allow/deny into category precedence logic
  - Verify allow/deny integration tests pass

- [ ] Create .noemoji.toml for project dogfooding
  - Depends on: Integrate allow/deny lists into detection pipeline
  - Current violations: 31 found in src/check.rs (test data) and src/lib.rs (documentation)
  - Create .noemoji.toml in repository root
  - Add allow list for legitimate Unicode in test data and documentation examples
  - Document configuration choices with comments explaining why
  - Verify noemoji runs cleanly on project codebase (exit code 0)
  - Commit .noemoji.toml to repository

### Output Formatting Tasks

- [ ] Update violation output to use Violation enum with categories
  - Depends on: Refactor InputSource::check() callback to use Violation
  - Write tests for Violation::Arrow output (file:line:col: fancy arrow: → [->])
  - Write tests for Violation::Emoji output without suggestion (file:line:col: emoji: 🚀)
  - Write tests for Violation::Checkmark output (file:line:col: checkmark: ✓ [[x]])
  - Write tests for Violation::MathSymbol output (file:line:col: math symbol: ≤ [<=])
  - Write tests for violations without replacements (category only, no brackets)
  - Format output as: {path}:{line}:{column}: {category}: {char} [{suggestion}]
  - Omit [{suggestion}] when replacement is None
  - Verify all violation output format tests pass

- [ ] Implement path normalization for output
  - Write tests for file path relative to CWD (src/main.rs not ./src/main.rs)
  - Write tests for removing ./ prefix from paths
  - Write tests for consistent path separator handling
  - Update InputSource::name() to return normalized relative path
  - Strip ./ prefix if present in relative paths
  - Verify path normalization test cases pass

- [ ] Add explicit stdout flushing after each violation
  - Depends on: Update violation output to use Violation enum with categories
  - Write tests for stdout flushing after each violation
  - Add stdout.flush() call after each violation output in main.rs
  - Verify flush behavior in tests

- [ ] Add --quiet/-q flag to CLI argument parser
  - Write tests for --quiet and -q flag parsing
  - Add quiet flag handling to lexopt argument parser
  - Store quiet mode state in CliCommand::Check variant
  - Verify flag parsing tests pass

- [ ] Implement quiet mode output suppression
  - Depends on: Add --quiet/-q flag to CLI argument parser, Update violation output to use Violation enum with categories
  - Write tests for violations detected but not displayed in quiet mode
  - Write tests for exit code 1 when violations found in quiet mode
  - Write tests for exit code 0 when clean in quiet mode
  - Write tests for exit code 2 on errors in quiet mode
  - Write tests for errors still output to stderr in quiet mode
  - Implement quiet mode logic in main.rs (skip violation output, preserve exit codes)
  - Verify all quiet mode behavior tests pass

### File Processing Tasks

- [ ] Implement TTY detection for stdin with no file arguments
  - Write tests for TTY on stdin showing help instead of hanging
  - Write tests for non-TTY stdin (pipe) processing normally
  - Write tests for explicit `-` argument processing stdin regardless of TTY
  - Add TTY detection using io::stdin().is_terminal() or atty crate
  - Show help text and exit 0 when stdin is TTY with no file arguments
  - Verify TTY detection test cases pass

- [ ] Add diagnostic log statements for configuration and detection
  - Depends on: Wire classifier pipeline into main detection flow
  - Add log::debug! for configuration discovery (which .noemoji.toml files loaded)
  - Add log::debug! for final merged configuration values at startup
  - Add log::debug! for per-violation detection explanations (character, code point, category)
  - Add log::debug! for exemption application when violations suppressed
  - Write tests verifying debug output with NOEMOJI_LOG=debug

### Git Integration Tasks

- [ ] Implement basic git command execution
  - Write tests for successful git command execution
  - Implement run_git_command() function for happy path
  - Return stdout/stderr on success
  - Verify git execution tests pass

- [ ] Add git error handling with structured errors
  - Write tests for "not a git repository" errors
  - Write tests for git binary not found
  - Create GitError type for structured error reporting
  - Return GitError for common failure cases
  - Verify error handling tests pass

- [ ] Add --staged flag to CLI argument parser
  - Write tests for --staged flag parsing
  - Add --staged flag handling to lexopt argument parser
  - Store staged mode state in CliCommand::Check variant
  - Verify flag parsing tests pass

- [ ] Implement git staged file discovery
  - Write tests for git diff --cached --name-only parsing
  - Write tests for no staged files returning empty list
  - Implement staged file discovery function
  - Verify staged file discovery tests pass

- [ ] Wire --staged flag into detection pipeline
  - Depends on: Add --staged flag to CLI argument parser, Implement git staged file discovery
  - Write tests for --staged processing only staged files
  - Wire staged file list into detection pipeline
  - Verify basic --staged integration tests pass

- [ ] Implement git diff --cached parser for staged line ranges
  - Depends on: Add git error handling with structured errors
  - Write tests for parsing unified diff format
  - Write tests for extracting line ranges from hunks
  - Write tests for files with multiple hunks
  - Implement diff parser returning staged line ranges per file
  - Verify diff parser tests pass

- [ ] Implement line range intersection for staged content filtering
  - Write tests for checking if violation line is within staged range
  - Write tests for multiple non-contiguous staged ranges
  - Implement is_line_in_staged_ranges() utility function
  - Verify intersection logic tests pass

- [ ] Filter violations to only report staged content
  - Depends on: Wire --staged flag into detection pipeline, Implement line range intersection
  - Write tests for violations in staged lines reported
  - Write tests for violations in unstaged lines suppressed
  - Write tests for partial staging (same file with staged and unstaged hunks)
  - Write tests for files with no staged changes
  - Apply line range filtering when processing staged files
  - Verify violation filtering tests pass

- [ ] Add error handling for --staged flag
  - Write tests for --staged with no git repo error
  - Write tests for --staged with no staged files (exit 0)
  - Handle git command errors gracefully
  - Verify --staged error handling tests pass

### Pre-commit Integration Tasks

- [ ] Create .pre-commit-hooks.yaml with hook definition
  - Depends on: Filter violations to only report staged content
  - Create .pre-commit-hooks.yaml in repository root
  - Define noemoji hook with:
    - id: noemoji
    - name: Check for Unicode violations (noemoji)
    - entry: noemoji --staged
    - language: system
    - pass_filenames: false (--staged handles files)
  - Write tests for hook definition YAML syntax
  - Verify hook works with pre-commit.com framework
  - Document installation methods in comments

### Documentation Tasks

- [ ] Add installation section to README with prerequisites
  - Depends on: Create GitHub release from v0.1.0 tag with Linux x64 binary
  - Document prerequisites: Rust toolchain 1.85+ (or latest stable)
  - Document cargo install --path . for local source installation
  - Document cargo build --release for development builds
  - Document binary download from GitHub releases
  - Add verification step: noemoji --version
  - Verify installation instructions are complete and accurate

- [ ] Add usage examples section to README
  - Depends on: Implement quiet mode output suppression, Wire --staged flag into detection pipeline
  - Document basic usage examples:
    - Single file: noemoji file.txt
    - Multiple files: noemoji src/*.rs
    - Stdin: cat file.txt | noemoji or noemoji < file.txt
    - Explicit stdin: noemoji file1.txt - file2.txt
  - Document exit codes: 0=clean, 1=violations found, 2=error
  - Document --staged flag for pre-commit workflows
  - Document --quiet flag for CI integration
  - Add reference to noemoji --help for complete options
  - Verify all examples execute correctly

- [ ] Add configuration section to README with .noemoji.toml examples
  - Depends on: Add detection category toggles, Integrate allow/deny lists into detection pipeline
  - Document .noemoji.toml discovery behavior (CWD -> parent -> root)
  - Document CLI flags override config file settings
  - Provide example configuration with:
    - Category toggles ([detection] section)
    - Allow/deny lists for specific characters
    - Inherit control (inherit = false stops upward search)
  - Document NOEMOJI_LOG environment variable
  - Verify configuration examples are valid

- [ ] Add pre-commit integration section to README
  - Depends on: Create .pre-commit-hooks.yaml, Create scripts/pre-commit example
  - Document .pre-commit-hooks.yaml usage with pre-commit framework
  - Provide example .pre-commit-config.yaml configuration
  - Document manual scripts/pre-commit hook installation
  - Explain exit code behavior in pre-commit context (1 blocks commit)
  - Verify pre-commit examples work correctly

- [ ] Add troubleshooting section to README
  - Document common error scenarios and resolutions
  - Document performance considerations for large codebases
  - Document debugging with NOEMOJI_LOG=debug
  - Verify troubleshooting guidance is accurate

- [ ] Add contributing guidelines to README
  - Document development setup and check.sh usage
  - Document testing requirements for contributions
  - Document code review and PR process
  - Reference MPL-2.0 license requirements
  - Verify contributing guidance is complete

### Quality Assurance Tasks

- [x] Create integration test infrastructure with assert_cmd
  - Add assert_cmd and predicates dev-dependencies to Cargo.toml
  - Create modular test files in tests/ (cli.rs, stdin.rs, config_discovery.rs, etc.)
  - Verify test infrastructure compiles and runs

- [x] Write integration test for file with violations (exit code 1)
  - Tests in tests/stdin.rs verify violations produce exit code 1
  - Tests cover stdin input with Unicode arrows producing violations

- [x] Write integration test for clean file (exit code 0)
  - Tests in tests/cli.rs and tests/stdin.rs verify clean files produce exit code 0
  - Tests use real project files (Cargo.toml, LICENSE) as clean input

- [x] Write integration test for multiple files with mixed results
  - Tests in tests/stdin.rs cover multiple inputs with mixed clean/violation/error results
  - Verify exit code 1 when any input has violations

- [x] Write integration test for file not found error (exit code 2)
  - Tests in tests/stdin.rs verify nonexistent files produce exit code 2
  - Tests verify error messages include missing filename

- [ ] Add noemoji to check.sh validation script
  - Depends on: Create .noemoji.toml for project dogfooding
  - Add noemoji execution to check.sh after existing Rust checks
  - Run noemoji on project source files: `noemoji src/ tests/ *.md *.toml`
  - Configure check.sh to fail (exit 1) if noemoji detects violations
  - Add clear output: "Running noemoji Unicode compliance check..."
  - Verify check.sh integration works correctly in CI

- [ ] Benchmark detection performance against grep on sample files
  - Create performance test with sample files of varying sizes
  - Benchmark noemoji processing time against `grep -P` for equivalent patterns
  - Verify noemoji is not noticeably slower than sequential grep
  - Document performance characteristics and any optimization needs
  - Ensure performance constraint "should not be noticeably slower than sequential grep" is met

- [ ] Verify release binary size meets <10MB constraint
  - Build release binary with `cargo build --release`
  - Strip binary: `strip target/release/noemoji`
  - Verify binary size is under 10MB per MVP constraint
  - Document actual binary size for baseline
  - Identify optimization opportunities if size exceeds constraint

- [ ] Verify dogfooding produces clean results
  - Run `./check.sh` locally and verify noemoji step passes
  - If violations found, analyze whether they're legitimate or false positives
  - Fix legitimate violations OR add to .noemoji.toml allow list with justification
  - Tune .noemoji.toml if needed for project's legitimate Unicode usage
  - Verify `./check.sh` passes completely with noemoji integration
  - Verify CI runs check.sh successfully with noemoji included

### CI and Release Tasks

- [ ] Run final validation before MVP release
  - Run cargo test to verify all tests pass
  - Run ./check.sh to verify all quality checks pass
  - Build and test release binary via "Create Linux x64 release build process" task
  - Review PLAN.md to confirm all MVP tasks are completed

- [ ] Create and publish v0.1.0 git tag
  - Depends on: Create Linux x64 release build process
  - Create git tag v0.1.0 with annotation: "MVP release: minimal end-to-end functionality"
  - Push tag to remote: git push origin v0.1.0

- [ ] Create GitHub release from v0.1.0 tag with Linux x64 binary
  - Create manual GitHub release for MVP from v0.1.0 tag
  - Attach stripped Linux x64 binary to release
  - Write release notes describing MVP features
  - Verify release binary downloads and runs correctly
