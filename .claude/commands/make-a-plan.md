---
name: make-a-plan
description: Create comprehensive implementation plans following design-first TDD approach
argument-hint: [requirements | file-path]
---

# Make a Plan Command

Create detailed implementation plans for features, refactoring, or improvements.
Plans follow a strict design-first test-driven development (TDD) approach.

Invoke the command: /make-a-plan [requirements | file-path]

If arguments are provided via $ARGUMENTS, they will be used as the planning requirements
(either inline text or a path to a markdown file with requirements).

## What This Command Does

This command invokes the `plan-guideline` skill to create a comprehensive implementation plan that includes:

1. **Goal Understanding** - Clear problem statement and success criteria
2. **Codebase Analysis** - Specific files to modify/create/delete (audit results, not audit tasks)
3. **Interface Design** - API changes and documentation structure
4. **Test Strategy** - Test cases designed before implementation
5. **Implementation Steps** - Ordered as Documentation → Tests → Implementation

## Output

The plan will be structured with:
- **Goal**: Problem statement, success criteria, and out-of-scope items
- **Codebase Analysis**: Files to modify/create/delete with line ranges
- **Interface Design**: New and modified interfaces
- **Test Strategy**: Test files and test cases
- **Implementation Steps**: Ordered steps with LOC estimates and dependencies
  - Step 1+: Documentation updates
  - Step N+: Test case creation/updates
  - Step M+: Implementation code
- **Complexity Estimate**: Total LOC and recommended approach (single session vs milestones)

## Next Steps

After the plan is created and approved:

1. **Create a GitHub issue** using the `/open-issue` command
   - The plan becomes the "Proposed Solution" section
   - Issue will be tagged as `[plan][tag]`

2. **Begin implementation** on a development branch
   - Follow the steps in order: Docs → Tests → Implementation
   - Use milestone commits for features > 800 LOC
   - Run tests at each milestone (accept temporary failures)
   - Only merge when all tests pass

## Key Principles

- **Concrete over vague**: No "audit the codebase" steps - audit happens during planning
- **LOC-based estimates**: Use lines of code, never time durations
- **Design-first TDD**: Documentation first, tests second, implementation last
- **Test-aware**: Tests must exist before writing implementation
- **Milestone-friendly**: For large features, plan includes milestone strategy with test progress tracking
