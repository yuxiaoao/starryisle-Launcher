# CI: known open items

One check is red on main while everything else passes.

Current state (run 44980338114): guardrails, frontend lint/tests/visual, i18n, the
theseus test suites and the Windows dual-installer packaging verification pass on
ubuntu, macOS and Windows. That packaging item was repaired in df13559 and b491bf7;
only the coverage gate below is still open.

## 1. Rust coverage gate

Measure coverage fails although every test suite passes.

The job runs the whole workspace under cargo-llvm-cov, so the likely cause is the
coverage threshold rather than a failing test.

Decision needed: either set the threshold to the level the fork actually holds today, or
add tests for the uncovered code. Lowering it silently weakens the gate, so the choice
should be explicit.
