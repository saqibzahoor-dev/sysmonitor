# Contributing

Bug reports, feature ideas, and pull requests are welcome.

## Reporting bugs

Open an issue with:
- SysMonitor version (from About menu)
- Windows version (`winver`)
- Steps to reproduce
- What you expected vs. what happened
- For sensor issues: whether you ran as admin

## Pull requests

- Keep changes focused — one fix or feature per PR
- Follow the existing code style
- Add tests for non-trivial Rust changes (`cd src-tauri && cargo test`)
- Frontend changes — run `npm run check` and `npm run build` before pushing
- Conventional commit prefix preferred (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`)

## Building from source

See [`README.md`](README.md) → **Development** section. Short version:

```powershell
# 1. Build C# sidecar (one-time, needs .NET SDK 8)
./scripts/build-sidecar.ps1

# 2. JS deps
npm install

# 3. Dev mode
cargo tauri dev

# 4. Production build (installer)
cargo tauri build
```

Requires Rust 1.77+ (MSVC), Node 18+, .NET SDK 8 with net48 target, VS 2022 Build Tools (C++ workload), Windows 10/11 (x64).

## Releases

Releases are tagged `vX.Y.Z` on `main`. The NSIS installer and MSI package are uploaded as release assets.
