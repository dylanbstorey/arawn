---
id: integrate-code-coverage-tooling
level: task
title: "Integrate code coverage tooling (llvm-cov) into CI"
short_code: "ARAWN-T-0287"
created_at: 2026-03-08T03:17:32.131835+00:00
updated_at: 2026-03-08T03:17:32.131835+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
initiative_id: NULL
---

# Integrate code coverage tooling (llvm-cov) into CI

## Objective

There is no code coverage measurement in the project. We have ~1,600 test functions across 22 crates but no way to know what percentage of code they actually exercise. Integrate `cargo-llvm-cov` into CI and local development to measure and track coverage over time.

### Priority
- [x] P3 - Low (observability, not blocking)
- **Size**: S

### Current Problems
- No way to quantify coverage — we're guessing where gaps are
- No coverage diff on PRs — regressions go unnoticed
- Can't set coverage thresholds or gates
- No per-crate coverage breakdown

## Acceptance Criteria

- [ ] `cargo llvm-cov` runs locally with `angreal test coverage` (or similar angreal task)
- [ ] CI generates coverage report on each push to main
- [ ] Coverage report published as CI artifact (HTML format)
- [ ] Per-crate coverage summary printed in CI output
- [ ] Coverage badge or summary in README (optional)
- [ ] Baseline coverage percentage established and documented

## Implementation Notes

### Local setup

```bash
# Install
rustup component add llvm-tools-preview
cargo install cargo-llvm-cov

# Run
cargo llvm-cov --workspace --html --output-dir coverage/
# or per-crate:
cargo llvm-cov -p arawn-server --html
```

### Angreal task

Add to `.angreal/task_test.py`:
```python
@angreal.command(name="coverage", about="Generate code coverage report")
@angreal.argument(name="open", long="open", is_flag=True, takes_value=False, help="Open report in browser")
def coverage(open=False):
    os.system("cargo llvm-cov --workspace --html --output-dir coverage/")
    if open:
        os.system("open coverage/html/index.html")
```

### CI integration

Add to `.github/workflows/ci.yml`:
```yaml
- name: Install cargo-llvm-cov
  uses: taiki-e/install-action@cargo-llvm-cov
- name: Generate coverage
  run: cargo llvm-cov --workspace --lcov --output-path lcov.info
- name: Upload coverage
  uses: actions/upload-artifact@v4
  with:
    name: coverage-report
    path: lcov.info
```

### Key files
- `.github/workflows/ci.yml` — Add coverage step
- `.angreal/task_test.py` — Add coverage command
- `.gitignore` — Add `coverage/` directory

### Dependencies
- None

## Status Updates

*To be added during implementation*