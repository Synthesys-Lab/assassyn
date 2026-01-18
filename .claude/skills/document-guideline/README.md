# Document Guideline Skill

This folder contains the document-guideline skill which instructs AI agents on documentation standards for the project.

## Purpose

The document-guideline skill provides comprehensive documentation standards that are:
1. **Enforced automatically** via pre-commit linting for structural requirements
2. **Referenced by other skills** for content quality guidance
3. **Validated through dogfooding** (the skill requires its own documentation)

## Files

- `SKILL.md` - Main skill definition with frontmatter and complete documentation standards
  - High-level design documentation guidelines (`docs/*`)
  - Folder README.md requirements (enforced by linting)
  - Source code .md file correspondence (enforced by linting)
  - Test documentation requirements (enforced by linting)
  - Milestone and linting interaction guidelines

## Integration

### With Pre-commit Linting

The skill is enforced by `scripts/lint-documentation.sh` which runs during the pre-commit hook:
- Checks all folders have README.md
- Checks all source files (`.py`, `.c`, `.cpp`, `.cxx`, `.cc`) have `.md` companions
- Checks all test files have documentation (inline or companion .md)

### With Other Skills

- **plan-guideline skill**: References documentation standards when creating implementation plans
  - Documentation steps always come first (design-first TDD)
- **milestone skill**: Uses documentation guidelines for incremental development
  - Allows `--no-verify` bypass during milestones for acceptable doc-code inconsistency

## Validation Approach

This skill uses **dogfooding** rather than explicit test cases:
- The linter validates its own documentation (`scripts/lint-documentation.md`)
- The skill folder requires this `README.md` (validated by the linter)
- Pre-commit hook integration tested during actual usage

## Usage

AI agents automatically reference this skill when:
- Creating implementation plans (ensures documentation steps are included)
- Writing code (reminds to create companion .md files)
- Running milestone commits (understands when linting bypass is acceptable)

The skill is not directly invocable by users - it provides guidelines that other skills reference.
