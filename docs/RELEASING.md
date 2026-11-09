# Cargo packages and releases

Status2026-11-09: preparation only. No crate has been registered or published by this work. Registry API checks returned404 for `craft-canvas`, `craft-canvas-model` and `craft-canvas-egui`; availability is transient and does not reserve a name. Publish real, useful packages, not placeholders. The current workspace declares0.1.0-alpha.1; this is not a registry release receipt.

The existing product has two packages: the GUI-independent `craft-canvas-model` and its `craft-canvas-egui` adapter. Retain that boundary. A `craft-canvas` facade is not currently selected merely to reserve a name. Users can consume the exact qualified Git revision while the API evolves, as shown in README.

## Qualification and publication sequence

CC-11 owns executable release setup. Before its closure:

1. Complete each package's description, README, documentation/repository/homepage metadata, keywords/categories and versioned internal dependency. Include the package license and applicable upstream notices in the actual archives. Verify examples against the real public API.
2. Run locked tests, strict Clippy, formatting and rustdoc on the supported hosted compiler. Run Cargo packaging/dry-run and inspect normalized manifests, included files and archive hashes. The adapter's registry package must resolve a real matching model version; a workspace path override alone cannot prove that.
3. For first publication, use the owner's verified crates.io account. Configure a narrowly scoped publish credential securely in the approved hosted environment; never paste it into chat or commit it. No registry credential is currently configured in this repository's GitHub secrets. Account availability is not inferred from GitHub authentication.
4. Publish the reviewed model version, verify registry propagation/ownership/source identity, then publish the matching adapter and qualify a minimal external consumer against the registry. Handle partial publication explicitly; never overwrite a version or claim both succeeded from one upload.
5. Bind a version tag, changelog, package checksums, hosted source/lock/compiler metadata and scoped release notes to the exact qualified source. Preserve existing tags/assets. Label engineering prereleases honestly; production/full parity requires CC-14.
6. After first publication, configure crates.io Trusted Publishing for the exact repository/workflow/environment where supported. Keep publication separate from pull-request jobs and ordinary qualification. A manual dispatch or approved version tag must identify the concrete reviewed source; never publish arbitrary workflow input without checking it against that source.

The [Cargo publishing guide](https://doc.rust-lang.org/cargo/reference/publishing.html) describes package inspection, immutable versions, account setup and dry-run. The [official trusted-publishing announcement](https://blog.rust-lang.org/2025/07/11/crates-io-development-update-2025-07/) describes the initial manual publication prerequisite and later OIDC publishing; recheck current registry support when implementing the workflow. Use official sources and pin third-party actions by reviewed commit. Do not install a local Cargo publishing tool or run local Rust builds for this task.

## Compatibility policy

Before1.0, focused prereleases may change API with explicit migration notes and exact consumer pins. This is not permission to discard stored data or silently change semantics. The WriterCraft compatibility profile remains required. Keep Rust/egui requirements explicit; do not raise them as an incidental effect of another graph library's latest release.

Each release records actual capabilities, known gaps, supported platforms and input evidence, not merely the eventual roadmap. Windows/Linux portable tests do not establish native GUI accessibility. Crates.io availability, a tag or a green compiler cannot replace independent native acceptance.
