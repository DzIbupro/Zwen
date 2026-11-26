# Third-party notice inventory

After updating a dependency or bundled asset, install the locked frontend dependencies, populate the Cargo cache for `x86_64-unknown-linux-gnu`, then run `node scripts/generate-third-party-notices.mjs` from the repository root. Commit the generated `frontend/license-data/` files with the dependency update. Run `node scripts/generate-third-party-notices.mjs --check` to verify them against the checkout. The generator runs offline against the installed dependencies and cached crate sources.

The inventory covers packages with rendered modules in the Vite production output, bundled KaTeX fonts and Lucide icons, and the Linux x86_64 desktop crate's normal dependency graph. The Cargo graph can include packages linked conditionally or eliminated at compile time. Build dependencies, procedural macros, and separately installed host packages such as WebKitGTK are outside that graph. Cargo license expressions come from upstream manifests, including older slash-separated expressions.

Most notices are read from the locked packages. The following sources fill gaps in the archives.

- `alloc-stdlib 0.3.0` shares the Dropbox license in `alloc-no-stdlib 3.0.0`. [Pinned upstream license](https://github.com/dropbox/rust-alloc-no-stdlib/blob/0a81fd6928ea3b33c8cd484aa4575d50ffb98012/LICENSE)
- `dlopen2 0.8.2` omits its license file from the crate. The local MIT text includes the copyright notices from the commit identified in `.cargo_vcs_info.json`. [Pinned upstream license](https://github.com/OpenByteDev/dlopen2/blob/cc80e4a0a90d499b677fdf7743699b4b3a43a989/LICENSE)
- `libappindicator-sys 0.9.0` shares both license files with `libappindicator 0.9.0` from the same upstream workspace and commit. [Pinned upstream source](https://github.com/tauri-apps/libappindicator-rs/tree/eafd1e3682a1247f595410266091e9684021cb6f)
- `selectors 0.38.0` has MPL-2.0 headers. The generator reuses the standard MPL-2.0 text available in DOMPurify. [Pinned source header](https://github.com/servo/stylo/blob/572ecba2d1600e7c3d490586692a209faf703baa/selectors/lib.rs) · [MPL-2.0](https://www.mozilla.org/MPL/2.0/)
- KaTeX font copyright and reserved names come from the shipped TTF name tables. The accompanying file contains the standard SIL Open Font License 1.1. [OFL-1.1](https://openfontlicense.org/open-font-license-official-text/)
- The pinned Lucide notice identifies Feather-derived portions under the MIT License. The supplemental text uses the copyright attribution in that notice with the Feather MIT terms. [Lucide 0.468.0 license](https://github.com/lucide-icons/lucide/blob/0.468.0/LICENSE) · [Feather license](https://github.com/feathericons/feather/blob/v4.29.0/LICENSE)
