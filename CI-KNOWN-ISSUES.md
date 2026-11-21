# CI: known open items

Two checks are red on main while everything else passes. Neither is caused by the
upstream sync; both predate it and had never been reached before, because earlier
guardrail failures stopped the run.

Current state (run 44908726440): guardrails, frontend lint/tests/visual, i18n and the
theseus test suites pass on ubuntu, macOS and Windows.

## 1. Windows dual-installer packaging verification

Build and test desktop application (windows-2025) fails at
Package and verify both Windows installers from one binary.

- The Rust tests on that job pass; the failure is after them, in packaging.
- ubuntu and macOS run the same tests green.
- The job log labels its steps UNKNOWN STEP, so the failing step's output could not be
  isolated with gh run view --job.

Next step: download the whole job log and grep for 
sis, portable and installer
instead of matching step names, then fix the packaging script or NSIS config and confirm
with a full StarryIsle build run (~80 minutes).

## 2. Rust coverage gate

Measure coverage fails although every test suite passes.

The job runs the whole workspace under cargo-llvm-cov, so the likely cause is the
coverage threshold rather than a failing test.

Decision needed: either set the threshold to the level the fork actually holds today, or
add tests for the uncovered code. Lowering it silently weakens the gate, so the choice
should be explicit.