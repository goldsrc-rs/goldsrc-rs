# Contributing to GoldSrc.rs

Thank you for your interest in contributing to **GoldSrc.rs**! We welcome contributions of all kinds: new features, bug fixes, documentation improvements, tests, and demo WASM plugins.

Please take a moment to review these guidelines before submitting code.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Branching Strategy](#branching-strategy)
- [Conventional Commits](#conventional-commits)
- [Development Setup](#development-setup)
- [Coding Standards & Safety](#coding-standards--safety)
- [Pull Request Workflow](#pull-request-workflow)

## Code of Conduct

This project and everyone participating in it is governed by the [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Branching Strategy

Our repository uses a GitFlow-inspired branching model:

- **`main`**: Production/stable releases only. Protected branch; direct commits are prohibited.
- **`dev`**: Active development integration branch. Protected branch; all feature and fix PRs must target `dev`.
- **`feature/<name>`**: New features and enhancements, branched from `dev` and merged back into `dev` via PR.
- **`fix/<name>`** or **`hotfix/<name>`**: Bug fixes, branched from `dev` (or `main` for critical production hotfixes).

```text
main   ───────────────────────────● (v1.0.0)
                                 /
dev    ──────●──────────●───────●─── (Active Development)
              \        /
feature/wasm   ●──────●             (Feature Branch)
```

## Conventional Commits

We strictly follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification. All commit messages must be formatted as:

```text
<type>(<optional-scope>): <description in imperative mood>

[optional body]

[optional footer(s)]
```

### Allowed Types

| Type | Description | Example |
| :--- | :--- | :--- |
| `feat` | A new feature or capability | `feat(wasm): add JIT compilation cache for wasmtime` |
| `fix` | A bug fix | `fix(standalone): correct entity dictionary indexing` |
| `docs` | Documentation changes only | `docs(readme): add troubleshooting section` |
| `refactor` | Code restructuring without changing behavior | `refactor(core): extract ECS storage into sparse set` |
| `perf` | A code change that improves performance | `perf(event-bus): optimize event dispatch loop` |
| `test` | Adding missing tests or correcting existing tests | `test(api): add unit tests for Vector3 operations` |
| `build` | Changes affecting build system or dependencies | `build(cargo): update wasmtime to v24.0` |
| `ci` | Changes to CI workflows and scripts | `ci(github): add dependabot configuration` |
| `chore` | Routine maintenance, tooling, or chore tasks | `chore: format codebase with cargo fmt` |

## Development Setup

### 1. Prerequisites

Ensure you have installed:

1. **Rust 1.80+** with required compilation targets:

   ```bash
   rustup default stable
   rustup target add i686-pc-windows-msvc      # Windows 32-bit HLDS target
   rustup target add i686-unknown-linux-gnu   # Linux 32-bit HLDS target
   rustup target add wasm32-unknown-unknown   # WASM plugin target
   ```

2. **Python 3.10+** (for automation CLI tools).
3. **C/C++ Build Tools**: MSVC x86 (Windows) or `gcc-multilib` / `g++-multilib` (Linux).

### 2. Setup Reference Headers

Clone the repository and run the setup tool to download reference SDKs:

```bash
git clone https://github.com/goldsrc-rs/goldsrc-rs.git
cd goldsrc-rs

# Download HLSDK / Metamod headers and generate local environment
python -m scripts setup
```

### 3. Build & Test

```bash
# Build workspace and plugins
python -m scripts build --backend all --release

# Run Rust unit and integration tests
cargo test --workspace

# Run pre-commit validation checks
python -m scripts pre-commit
```

## Coding Standards & Safety

### 1. Language Policy

- **User Documentation & Codebase**: All code, comments, docstrings, commit messages, PR descriptions, and documentation **must be written strictly in English**.

### 2. Safety & FFI Boundaries

- **No Panics across FFI**: Never allow raw Rust panics to unwind across C-ABI / engine boundaries. Always wrap extern entry points in `std::panic::catch_unwind`.
- **`// SAFETY:` Comments**: Every `unsafe` block or function must be preceded by a clear `// SAFETY: <rationale>` comment explaining why the invariants are upheld.
- **Defensive Resource Management**: Always avoid deadlocks in re-entrant engine callbacks. Favor narrow lock scopes and re-entrant lock patterns where engine callbacks may trigger recursively.

### 3. Formatting & Linting

Before creating a commit, ensure your code passes:

```bash
# Check formatting
cargo fmt --all -- --check

# Check clippy lints (0 warnings allowed)
cargo clippy --workspace --all-targets -- -D warnings
```

## Pull Request Workflow

1. Create a feature branch from `dev`:

   ```bash
   git checkout dev
   git pull origin dev
   git checkout -b feature/my-awesome-feature
   ```

2. Make your changes and commit with Conventional Commits.
3. Verify all automated checks pass locally:

   ```bash
   python -m scripts pre-commit
   ```

4. Push your branch to your fork:

   ```bash
   git push origin feature/my-awesome-feature
   ```

5. Open a Pull Request targeting `dev`.
6. Provide a structured PR description using the following template:

```markdown
### Summary
Brief description of what this PR does and why.

### Changes
- List key architectural or functional changes
- Mention any new dependencies or configuration options

### Verification
- [ ] `cargo fmt --check` passed
- [ ] `cargo clippy --all-targets` passed (0 warnings)
- [ ] `cargo test --workspace` passed
- [ ] Tested locally on HLDS (Windows / Linux)
```
