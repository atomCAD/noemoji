#!/bin/bash
set -euo pipefail

# Check for required binaries
if ! command -v markdownlint-cli2 >/dev/null 2>&1; then
    echo "Error: markdownlint-cli2 is not installed" >&2
    exit 1
fi
MARKDOWNLINT="markdownlint-cli2"

if ! command -v shellcheck >/dev/null 2>&1; then
    echo "Error: shellcheck is not installed" >&2
    exit 1
fi
SHELLCHECK="shellcheck"

# Check for markdown files with syntax issues
git ls-files --cached --others --exclude-standard '*.md' | xargs -r "$MARKDOWNLINT"

# Check for shell scripts with syntax issues
git ls-files --cached --others --exclude-standard '*.sh' | xargs -r "$SHELLCHECK"

# Check for trailing whitespace
if whitespace_output=$(git ls-files --cached --others --exclude-standard '*' | xargs -r grep -Hn '[[:blank:]]$' 2>/dev/null); then
    while IFS= read -r line; do
        printf '%s <-- trailing whitespace\n' "$line"
    done <<< "$whitespace_output" >&2
    exit 1
fi

# Rust checks
cargo check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
RUSTDOCFLAGS="-D warnings -D missing-docs" cargo doc --workspace --no-deps

# Find and run all check.sh files in the workspace (except this one)
# I heard you like check.sh, so I put some check.sh in your check.sh. Check-ception.
find . -name "check.sh" -type f ! -path "./check.sh" -print0 2>/dev/null | sort -z | while IFS= read -r -d '' script; do
    # Check if the script is executable
    if [ ! -x "$script" ]; then
        echo "Error: \"$script\" is not executable" >&2
        echo "Run: chmod +x \"$script\"" >&2
        exit 1
    fi

    # Run the check script
    "$script"
done

# EOF
