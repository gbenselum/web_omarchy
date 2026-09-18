# Phase 13: Code Review & PR Standards

## Agent Assignment: Agent 13 (Code Review Lead)

### Objective
Enforce rigorous, structured code reviews to ensure all contributions align with Omarchy's architectural goals, security posture, and coding style.

### Key Requirements
1. **PR Structure Enforcement:**
   - Ensure every Pull Request follows a strict template, clearly answering:
     - *The problem:* What is being fixed/added?
     - *The change:* How does this PR solve it?
     - *Testing:* Detailed list of test suites run and assertions verified (e.g., `test/cli/suite.sh: 12/12`).
     - *Context:* Links to relevant issues or architectural documents.
2. **Review Criteria:**
   - **Security:** Ensure no privileges are escalated unnecessarily. Reject any code that runs the main web process as root.
   - **Performance:** Reject UI code that relies on heavy frameworks (React, PatternFly) in favor of the lightweight Web TUI standard.
   - **Style:** Enforce the Omarchy scripting style (Bash 5 conditionals, specific indentations, standard command prefixes like `omarchy-`).
3. **Automated Review Tools:**
   - Set up and manage automated linters (e.g., `shellcheck`, `eslint`, Rust `clippy`) to catch basic violations before human or AI review.

### Deliverables
- Comprehensive `.github/PULL_REQUEST_TEMPLATE.md`.
- Linter configurations (`.shellcheckrc`, `.eslintrc`, etc.).
- A documented Code Review checklist appended to `AGENTS.md`.
