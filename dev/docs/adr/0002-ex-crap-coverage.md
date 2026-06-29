# 0002 — CRAP Scores over Plain Coverage to Prevent Coverage Blindness

## Status

Accepted

## Context

Standard code coverage metrics (line, branch, or function coverage) create a
perverse incentive: teams chase a percentage target, so they write trivial
tests for trivial code while complex, high-risk paths remain untested. This is
known as **coverage blindness** — the number looks good while the real risk
hides in uncovered complexity.

Traditional coverage gates (e.g., "80% line coverage or CI fails") amplify the
problem. They encourage testing getters, setters, and generated code over the
domain logic that actually breaks in production.

CRAP (Change Risk Anti-Patterns) scores, originally proposed by
Alberto Savoia, combine cyclomatic complexity with test coverage into a single
metric:

```
CRAP(n) = complexity^2 × (1 - coverage)^3 + complexity
```

A function with complexity 5 and 100% coverage scores 5 (safe).
A function with complexity 10 and 0% coverage scores 1010 (unsafe).
A function with complexity 10 and 80% coverage scores 18 (borderline).

This creates two paths to a passing score: write meaningful tests or simplify
the code. Both improve the codebase.

## Decision

We use `ex_crap` to enforce a maximum CRAP score of **30** per function in CI,
rejecting any PR that introduces uncovered complexity.

### Configuration

- All functions scored via `mix crap` against persisted coverage data
- Default threshold of 30 (historical CRAP convention)
- Enforced in CI as a separate `coverage` job, not a pre-commit hook
  (coverage runs are slower and should not block local commits)
- The `mix coverage` alias bundles test export and crap scoring:
  `mix test --cover --export-coverage default && mix crap`

### What this replaces

No existing gate is removed. The CRAP score is an additional guard that
sits alongside `mix test`, `mix credo`, and `mix format`. It does not replace
coverage percentage targets — it makes them unnecessary by targeting the
specific intersection of complexity and untested code.

### What this does not do

- It does not measure test quality (assertions, edge cases, integration coverage)
- It does not replace code review
- It does not enforce coverage on configuration, generators, or trivial delegates
- It penalises functions with no coverage entry as 0% covered, so dead or
  unused code will also surface as CRAP violations

## Consequences

- **Positive**: Incentivises testing of complex domain logic and simplification
  of over-engineered code
- **Positive**: Eliminates the "80% coverage of nothing" treadmill — trivial
  functions auto-pass because low complexity keeps the score low
- **Positive**: Provides a clear, actionable queue for refactoring: sort by
  CRAP score, fix the worst first
- **Negative**: Adds ~10–15s to CI per coverage run
- **Negative**: False positives possible on legitimately complex functions
  (e.g., a parser with many branches that is exhaustively tested) — threshold
  can be bumped per-function with `@lint` or inline overrides in a future pass
- **Negative**: Requires persisted coverage data; `mix test --cover` alone does
  not leave an importable file (must use `--export-coverage`)
