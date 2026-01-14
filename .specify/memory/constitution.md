<!--
  ============================================================================
  SYNC IMPACT REPORT
  ============================================================================
  Version Change: 0.0.0 → 1.0.0 (MAJOR - initial constitution ratification)

  Modified Principles: N/A (initial version)

  Added Sections:
  - Principle I: Test-First Development (NON-NEGOTIABLE)
  - Principle II: Security by Design
  - Principle III: Code Quality & Readability
  - Principle IV: Defensive Programming
  - Section: Security Requirements
  - Section: Development Workflow
  - Section: Governance

  Removed Sections: N/A (initial version)

  Templates Requiring Updates:
  - .specify/templates/plan-template.md: ✅ Already has Constitution Check section
  - .specify/templates/spec-template.md: ✅ Compatible (user stories with acceptance tests)
  - .specify/templates/tasks-template.md: ✅ Already supports test-first workflow
  - .specify/templates/checklist-template.md: ✅ Compatible (can generate security checklists)
  - .specify/templates/agent-file-template.md: ✅ No changes needed

  Follow-up TODOs: None
  ============================================================================
-->

# iamawrapper Constitution

## Core Principles

### I. Test-First Development (NON-NEGOTIABLE)

All feature development MUST follow Test-Driven Development (TDD) methodology:

- **Tests MUST be written before implementation code**
- **Red-Green-Refactor cycle is mandatory**:
  1. Write a failing test that defines expected behavior
  2. Implement the minimum code to make the test pass
  3. Refactor while keeping tests green
- **No production code without corresponding tests**
- **Test coverage MUST include**: unit tests, integration tests, and contract tests where applicable
- **Tests MUST fail first** - if a test passes before implementation, it is invalid

**Rationale**: TDD ensures code correctness, provides living documentation, enables safe refactoring, and catches regressions early. Writing tests first forces clear thinking about requirements and API design.

### II. Security by Design

Security is not an afterthought but a foundational requirement:

- **Security MUST be considered at design phase**, not bolted on later
- **Input validation is mandatory** at all system boundaries (user input, external APIs, file I/O)
- **OWASP Top 10 vulnerabilities MUST be actively prevented**:
  - SQL injection
  - XSS (Cross-Site Scripting)
  - CSRF (Cross-Site Request Forgery)
  - Insecure deserialization
  - Command injection
  - Path traversal
  - And other injection attacks
- **Principle of least privilege** applies to all components
- **Secrets MUST never be committed** to version control
- **Dependencies MUST be audited** for known vulnerabilities

**Rationale**: Security breaches are costly in reputation, legal liability, and user trust. Proactive security is orders of magnitude cheaper than reactive incident response.

### III. Code Quality & Readability

Beautiful code is maintainable code:

- **Code MUST be self-documenting** through clear naming and structure
- **Functions SHOULD do one thing well** (Single Responsibility Principle)
- **Avoid premature optimization** - clarity over cleverness
- **Consistent code style is mandatory** - use automated formatters and linters
- **Dead code MUST be deleted**, not commented out
- **Magic numbers and strings MUST be named constants**
- **Comments explain "why", not "what"** - code itself shows what
- **Deprecated functions/APIs MUST NOT be used** - always use the recommended replacement; if deprecated code exists, it MUST be updated when encountered

**Rationale**: Code is read far more often than written. Clear, beautiful code reduces cognitive load, accelerates onboarding, and prevents bugs from hiding in complexity. Using deprecated APIs creates technical debt and risks breaking changes in future updates.

### IV. Defensive Programming

Assume inputs are hostile and systems fail:

- **Validate all external inputs** before processing
- **Handle errors explicitly** - no silent failures
- **Fail fast and loudly** - surface problems immediately
- **Log security-relevant events** with appropriate detail
- **Use type systems and static analysis** to catch errors at compile time
- **Sanitize data** before output (HTML encoding, SQL parameterization, etc.)

**Rationale**: Defensive programming transforms potential security vulnerabilities and runtime crashes into handled conditions and informative error messages.

## Security Requirements

All code in this project MUST adhere to these security requirements:

- **Authentication**: Use proven authentication mechanisms; never roll your own crypto
- **Authorization**: Implement proper access controls; verify permissions before actions
- **Data Protection**: Encrypt sensitive data at rest and in transit
- **Logging**: Log security events but never log sensitive data (passwords, tokens, PII)
- **Error Handling**: Never expose stack traces or internal details to end users
- **Dependencies**: Keep dependencies updated; monitor for CVEs
- **Configuration**: Secure defaults; fail closed, not open

## Development Workflow

All development work MUST follow this workflow:

1. **Understand Requirements**: Read and analyze the specification thoroughly
2. **Write Tests First**: Create failing tests that define expected behavior
3. **Get Test Approval**: Tests MUST be reviewed before implementation begins
4. **Watch Tests Fail**: Verify tests fail for the right reasons (Red)
5. **Implement Minimum Code**: Write just enough code to pass tests (Green)
6. **Refactor**: Improve code quality while keeping tests green
7. **Security Review**: Check for vulnerabilities before merge
8. **Code Review**: All changes require peer review

**Quality Gates**:
- All tests MUST pass
- No new security vulnerabilities
- Code style checks MUST pass
- Test coverage MUST not decrease

## Governance

This constitution is the supreme authority for development practices in this project:

- **Constitution supersedes** all other practices, preferences, and conventions
- **Amendments require**:
  1. Written proposal with rationale
  2. Team review and discussion
  3. Documentation of the change
  4. Update to CONSTITUTION_VERSION
- **Compliance verification**: All PRs and code reviews MUST verify compliance with these principles
- **Violations MUST be justified**: Any deviation from these principles requires explicit documentation of why it was necessary and what alternatives were rejected

**Version**: 1.1.0 | **Ratified**: 2026-01-09 | **Last Amended**: 2026-01-14
