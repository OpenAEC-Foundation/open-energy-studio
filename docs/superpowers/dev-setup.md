# Local Development Setup — Notes

## Rust toolchain on Windows

The Rust workspace targets `x86_64-pc-windows-msvc`. Linking requires **MSVC
`link.exe`** from the Visual Studio Build Tools. On a machine without VS Build
Tools, `cargo build` fails with:

```
link: extra operand 'C:\\...\\build_script_build.*.rcgu.o'
Try 'link --help' for more information.
```

The error is misleading — it comes from GNU coreutils `link` (shipped with
git for Windows) being picked up instead of MSVC's `link.exe`.

### Fix

Install **Visual Studio Build Tools 2022** (or VS Community), with the
"Desktop development with C++" workload. The
[official rustup docs](https://rust-lang.github.io/rustup/installation/windows.html)
have direct download links.

After install, open a fresh terminal — `link.exe` should resolve via
`where.exe link.exe`, and `cargo build --workspace` succeeds.

### CI

GitHub Actions runners have MSVC Build Tools pre-installed; no manual setup
needed for `release.yml`.

## Frontend

```sh
npm install
npm run tauri:dev
```

`@openaec/ui` is fetched from GitHub on `npm install`. If the dependency cannot
be resolved (firewall, private repo), the dev shell still loads — only the
shared UI primitives will be missing.

## Submodule

`vendor/crates-warehouse` is a git submodule. After cloning the parent repo:

```sh
git submodule update --init --recursive
```

Pinned at commit `e6a9edf` of
`https://github.com/OpenAEC-Foundation/crates-warehouse`.

To pull a newer warehouse:

```sh
cd vendor/crates-warehouse
git fetch && git checkout <new-sha>
cd -
git add vendor/crates-warehouse
git commit -m "chore(deps): bump crates-warehouse to <new-sha>"
```
