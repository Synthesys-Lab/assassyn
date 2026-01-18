# Review Standard Skill

This directory contains the review-standard skill for comprehensive code review of changes.

## Purpose

The review-standard skill provides AI agents with systematic guidance for reviewing code changes
before merging to main. It ensures quality, consistency, and adherence to project documentation
and code reuse standards.

## Integration

This skill is invoked by the `/code-review` command and integrates with:
- `document-guideline` skill - References documentation standards for review criteria
- `scripts/lint-documentation.sh` - Uses for structural documentation validation
- Git and GitHub CLI - For accessing change diffs and repository context

## Usage

See `SKILL.md` for complete review process and standards.
