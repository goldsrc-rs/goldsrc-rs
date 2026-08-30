### Summary
<!-- Provide a clear and concise overview of the purpose and motivation of this pull request. -->

### Changes
<!-- Detail the key architectural and codebase modifications grouped by crate or component. -->
- **Component / Crate**:
  - Detailed change item...
  - Detailed change item...

### Related Issues
<!-- Link related issues or discussions if applicable (e.g. Closes #123, Fixes #456). -->

### Verification Checklist
- [ ] Code formatting verified with `cargo fmt --check` (or auto-staged via pre-commit).
- [ ] Linters passed with 0 errors/warnings (`cargo clippy --workspace --all-targets -- -D warnings`).
- [ ] All automated workspace unit and integration tests pass (`cargo test --workspace`).
- [ ] WASM demo plugins compile and optimize cleanly (`python -m scripts build --wasm`).
- [ ] Conventional Commit guidelines and English code artifact rules adhered to.
- [ ] Verified live deployment or integration test on GoldSrc / HLDS runtime.
