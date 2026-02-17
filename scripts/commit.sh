#!/usr/bin/env bash
# Stage all changes and check for .gitignore candidates
# Usage: ./scripts/commit.sh [push]
# If "push" argument is provided, outputs PUSH_AFTER_COMMIT instruction

set -euo pipefail

PUSH_AFTER_COMMIT="${1:-}"

# Patterns that suggest a file should be in .gitignore
GITIGNORE_PATTERNS=(
    '\.log$'
    '\.tmp$'
    '\.cache$'
    '\.pyc$'
    '\.pyo$'
    '\.o$'
    '\.a$'
    '\.so$'
    '\.dylib$'
    '\.env$'
    '\.env\.local$'
    '\.DS_Store$'
    '\.swp$'
    '\.swo$'
    'node_modules/'
    '__pycache__/'
    '\.venv/'
    'venv/'
    '\.idea/'
    'dist/'
    'build/'
    'coverage/'
    '\.coverage$'
    'credentials\.'
    'secrets\.'
    '\.key$'
    '\.pem$'
    '\.p12$'
)

# Get untracked files into an array
# shellcheck disable=SC2312 # failure inside process substitution is fine; set -e handles it
mapfile -t untracked < <(git ls-files --others --exclude-standard)

# Check untracked files for gitignore candidates BEFORE staging
if [[ ${#untracked[@]} -gt 0 ]]; then
    CANDIDATES=()
    for file in "${untracked[@]}"; do
        for pattern in "${GITIGNORE_PATTERNS[@]}"; do
            if [[ "${file}" =~ ${pattern} ]]; then
                CANDIDATES+=("${file}")
                break
            fi
        done
    done

    if [[ ${#CANDIDATES[@]} -gt 0 ]]; then
        echo "GITIGNORE_CANDIDATES:"
        printf '%s\n' "${CANDIDATES[@]}"
        echo ""
        echo "These untracked files may belong in .gitignore."
        exit 2
    fi
fi

# No gitignore candidates - stage all changes
git add -A
echo "Staged changes:"
git status --short

# Output push instruction if requested
if [[ "${PUSH_AFTER_COMMIT}" == "push" ]]; then
    echo ""
    echo "PUSH_AFTER_COMMIT"
fi
