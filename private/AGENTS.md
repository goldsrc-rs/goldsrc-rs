# AGENTS

## System Prompt Instructions

1. **НЕ занимайтесь ручным реверс-инжинирингом.** Вся структура классов, виртуальные функции и оффсеты уже известны. Используйте предоставленные референсы: заголовки **HLSDK**, исходники **ReHLDS**, конфиги **hamdata.ini** и C# интероп из **GoldSrcMod.Net**.
2. **Используйте автоматизацию.** Для связи с C++ кодом используйте ТОЛЬКО `rust-bindgen`. Не пишите структуры `sys` руками.
3. **Unsafe-код.** Любой `unsafe` блок должен иметь исчерпывающий комментарий `// SAFETY: ...`, объясняющий, почему указатель или преобразование типов валидно.
4. **Конвенции вызовов.** Движок GoldSrc работает на 32-битной архитектуре (i686). Учитывайте разницу ABI между Windows (`__stdcall`, `__thiscall`) и Linux (`__cdecl`, System V). Для этого используйте правильные атрибуты `extern "C"` / `extern "system"` / `extern "thiscall"`.

## Repository Management

### Initialization

For repository initialization use `gh` and `git` commands:
```bash
gh repo create <OWNER>/<PROJECT-NAME> --public --description "..." --homepage "..." --source=. --push
```

### Branching Strategy

- `main` — stable releases only. Protected branch: requires PR with 1 approval, linear history, no force pushes, no deletions.
- `dev` — active development. Protected branch: requires PR with 1 approval.
- `feature/<name>` — feature branches. Branch from `dev`, merge back via PR. Delete after merge.

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `build`, `ci`.

Examples:
- `feat(goldsrc-sys): add bindgen-generated FFI for edict_t`
- `fix(goldsrc-api): correct IPlayer trait method signature`
- `chore: update .gitignore for Rust target`

### Pull Requests

- All changes go through PRs — no direct pushes to `main` or `dev`.
- PRs must target `dev` (or `main` for releases).
- PRs require at least 1 approval and must pass CI.
- Squash-and-merge is the default merge strategy.
- Delete branch after merge.

### Releases

- Tagged on `main` branch with semantic versioning (`v1.0.0`).
- Release notes generated from PR titles since last release.

### CI/CD

- GitHub Actions runs on every push and PR.
- Builds for `i686-pc-windows-msvc` and `i686-unknown-linux-gnu`.
- Runs `cargo fmt`, `cargo clippy`, `cargo test`.

### Branch Protection Rules

| Branch | Required PR | Required Status Checks | Required Linear History | Allow Force Pushes | Allow Deletions |
|--------|-------------|----------------------|------------------------|-------------------|-----------------|
| `main` | Yes (1 approval) | Yes | Yes | No | No |
| `dev` | Yes (1 approval) | No | No | No | No |

### References

All reference repositories are downloaded via `python3 scripts/setup.py` and gitignored.

- `references/hlsdk/` — HLSDK headers. Required for `bindgen` at build time.
- `references/metamod-r/` — metamod-r headers. Required for `bindgen` at build time.
- `references/rehlds/` — ReHLDS source. For reference only.
- `references/goldsrcmod-net/` — GoldSrcMod.Net source. For reference only.

### Agent Work

All agent work, notes, and local documentation go in `private/`. Never commit agent traces to the main repo.
