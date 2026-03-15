# Development Workflow

## Test-Driven Development (TDD)

Each feature task is split into two phases:
1. **Write Tests**: Define expected behavior via test cases (unit + integration)
2. **Implement**: Write the feature to make tests pass

### Test Coverage Requirement
- Minimum: 80% code coverage
- Measured via: `cargo tarpaulin --out Html --output-dir coverage`
- CI enforces: `cargo test && cargo tarpaulin --fail-under 80`

## Commit Strategy

Commit after each completed task (not per phase within a task):

```bash
# After completing a full task (tests + implementation)
git add src/
git commit -m "feat(module): brief description"
```

Commit messages follow Conventional Commits:
- `feat:` New feature
- `fix:` Bug fix
- `refactor:` Code restructuring (no functional change)
- `docs:` Documentation only
- `test:` Test additions or updates
- `chore:` Tooling, dependencies, cleanup

## Task Completion Protocol

For each task in the plan:
1. Write tests first (or update existing)
2. Implement feature/fix
3. Verify: `cargo test`, `cargo clippy`, `cargo fmt`, coverage check
4. Create commit with Conventional Commits message
5. Mark task as `[x]` in plan.md

## Code Review Checklist

Before marking a task complete:
- [ ] All tests pass: `cargo test --all`
- [ ] No clippy warnings: `cargo clippy --all-targets -- -W clippy::all`
- [ ] Code formatted: `cargo fmt --check`
- [ ] Coverage acceptable: `cargo tarpaulin --fail-under 80`
- [ ] Doc comments on public items
- [ ] No unsafe code (or justified SAFETY comments)
- [ ] Error handling uses `Result` and `?`

## Phase Completion Verification Protocol

At the end of each phase:
1. Run full test suite: `cargo test --all --all-features`
2. Generate coverage report: `cargo tarpaulin --out Html`
3. Run clippy with strict lints: `RUSTFLAGS="-W clippy::all" cargo clippy --all-targets`
4. Verify no merge conflicts or uncommitted changes
5. Record phase completion in Git Notes or commit message

Example Git Note:
```
Phase: Core Rendering Engine [COMPLETE]
- Tests: 42 passing (100% coverage for render_engine module)
- Clippy: 0 warnings
- Perf: avg render time 0.8s (baseline met)
```

## Continuous Integration

All commits trigger CI checks:
- `cargo check` — Compilation check
- `cargo test --all` — All tests
- `cargo clippy --all-targets -- -W clippy::all` — Linting
- `cargo fmt --check` — Format check
- `cargo tarpaulin --fail-under 80` — Coverage

No merge until all checks pass.
