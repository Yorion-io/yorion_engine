# Publishing YorionEngine

Every release goes to the GitHub release page and to five package managers. All of them ship the same version number, taken from `engine/Cargo.toml`.

| Ecosystem | Package | Install | Built by | Uploaded by |
| --- | --- | --- | --- | --- |
| Rust | `yorion_engine` on crates.io | `cargo add yorion_engine` | — | `publish.yml` → `crates-io` |
| JavaScript / TypeScript | `@yorion/engine` on npm | `npm i @yorion/engine` | `scripts/build-wasm.sh` + `scripts/package-npm.sh` | `publish.yml` → `npm` |
| Python | `yorion-engine` on PyPI | `pip install yorion-engine` | `python/pyproject.toml` (maturin + UniFFI) | `publish.yml` → `pypi` |
| Swift (iOS, macOS) | Swift Package Manager, this repo | Xcode → Add Package → repo URL | `scripts/package-apple-spm.sh` | `release.yml` (commits `Package.swift`, attaches the XCFramework) |
| Kotlin (Android) | `io.yorion:yorion-engine` on Maven Central | `implementation("io.yorion:yorion-engine:<v>")` | `android/` + `scripts/build-android.sh` | `publish.yml` → `maven-central` |
| Dart / Flutter | pub.dev | — | **Not set up yet**, see [the end of this guide](#flutter--pubdev-not-automated-yet) | — |

## How a release flows

1. A push to `main` runs **`release.yml`**. It runs the tests, builds the WASM, Apple and Swift Package Manager artifacts, bumps the version in `engine/Cargo.toml` and `engine/Cargo.lock`, writes `Package.swift` for that version, tags `vX.Y.Z` and creates the GitHub release.
2. The new release triggers **`publish.yml`**. It checks out the tag, confirms the tag matches the crate version, and publishes to every registry whose switch is on.
3. **`package-check.yml`** builds the crates.io, npm and PyPI packages on every pull request that touches the engine or the packaging. It uploads nothing.

Each registry has its own switch, a repository variable. A registry that isn't switched on is skipped, not failed, so you can turn them on one at a time as you finish their setup.

| Variable | Turns on |
| --- | --- |
| `PUBLISH_CRATES_IO` | crates.io |
| `PUBLISH_NPM` | npm |
| `PUBLISH_PYPI` | PyPI |
| `PUBLISH_MAVEN_CENTRAL` | Maven Central |

Set each one to `true` under **Settings → Secrets and variables → Actions → Variables**.

> `release.yml` creates the GitHub release with the `RELEASE_TOKEN_PAT` secret. Keep it that way: GitHub does not start other workflows for releases made with the default `GITHUB_TOKEN`, so `publish.yml` would never run.

### Trying it without uploading

Go to **Actions → Publish Packages → Run workflow**, enter an existing tag such as `v0.5.0`, and leave **dry run** ticked. Every package gets built and checked, and nothing is uploaded.

---

## One-time manual setup

Only a maintainer can do these steps. They involve creating accounts, proving domain ownership and adding secrets. Work through them in this order.

### 0. GitHub

1. Create four environments under **Settings → Environments**: `crates-io`, `npm`, `pypi` and `maven-central`. The names must match exactly, because each registry's trusted-publisher entry checks the environment name. Adding yourself as a required reviewer is optional and gives you a manual approval gate before anything is uploaded.
2. Leave the four `PUBLISH_*` variables unset for now. You set each one when you reach its section below.

### 1. crates.io

crates.io only lets you set up trusted publishing for a crate that already exists, so the first version is uploaded by hand.

1. Sign in at <https://crates.io> with GitHub, verify your email under **Account Settings**, and create an API token with the `publish-new` scope.
2. Publish the first version from a clean checkout of a release tag:
   ```bash
   cd engine
   cargo login            # paste the token
   cargo publish --dry-run
   cargo publish
   ```
3. On the crate page, go to **Settings → Trusted Publishing → Add** and enter:
   - Repository owner: `Yorion-io`
   - Repository name: `yorion_engine`
   - Workflow filename: `publish.yml`
   - Environment: `crates-io`
4. Delete the API token from step 1.
5. Set `PUBLISH_CRATES_IO=true`.

Optionally, add a second owner (`cargo owner --add <github-user>`) so the crate isn't tied to a single account.

### 2. npm

npm also requires the package to exist before trusted publishing can be enabled.

1. Create an npm account and turn on two-factor authentication.
2. Create the organization **`yorion`** at <https://www.npmjs.com/org/create>. The free plan is enough for public packages. This reserves the `@yorion` scope.
3. Publish the first version by hand:
   ```bash
   ./scripts/build-wasm.sh          # needs: rustup target add wasm32-unknown-unknown
                                    #        cargo install wasm-bindgen-cli --version 0.2.122
   ./scripts/package-npm.sh
   cd dist/npm
   npm login
   npm publish --access public
   ```
4. On npmjs.com, go to **@yorion/engine → Settings → Trusted publishing → GitHub Actions** and enter:
   - Organization or user: `Yorion-io`
   - Repository: `yorion_engine`
   - Workflow filename: `publish.yml`
   - Environment: `npm`
5. Recommended: on the same page, choose **Require two-factor authentication and disallow tokens**, so that only the workflow can publish.
6. Set `PUBLISH_NPM=true`.

Packages published through trusted publishing get a provenance statement automatically. npm only generates one when the GitHub repository is public.

### 3. PyPI

PyPI supports a *pending* publisher, so it needs no manual first upload.

1. Create an account at <https://pypi.org> and turn on two-factor authentication.
2. Go to <https://pypi.org/manage/account/publishing/>, open **Add a new pending publisher → GitHub**, and enter:
   - PyPI project name: `yorion-engine`
   - Owner: `Yorion-io`
   - Repository name: `yorion_engine`
   - Workflow name: `publish.yml`
   - Environment name: `pypi`
3. Set `PUBLISH_PYPI=true`. The next release creates the project.

Optionally, rehearse on TestPyPI (<https://test.pypi.org>) first by adding the same pending publisher there and temporarily adding `repository-url: https://test.pypi.org/legacy/` to the `pypa/gh-action-pypi-publish` step.

### 4. Swift Package Manager

There is no registry to sign up for, because SwiftPM installs straight from this git repository.

- On the first release after this change, `release.yml` writes `Package.swift` at the repo root and commits it with the tag. From then on, iOS and macOS developers add `https://github.com/Yorion-io/yorion_engine` in Xcode and pick a version.
- The binary comes from the release asset `yorion_engineFFI.xcframework.zip`. `Package.swift` pins its SHA-256, so never re-upload or replace that asset on an existing release.
- Optional: list the package on the Swift Package Index (<https://swiftpackageindex.com/add-a-package>). The repo needs to be public, with at least one tagged release that has a `Package.swift`.
- CocoaPods is deliberately not supported. Its central index becomes read-only on 2 December 2026.

### 5. Maven Central (Android)

1. Sign in at <https://central.sonatype.com> (a GitHub login works).
2. Go to **Namespaces → Add namespace**, enter `io.yorion`, and add the TXT record it shows to the `yorion.io` DNS. Then click **Verify**. Publishing is blocked until the namespace is verified.
3. Go to **Account → Generate User Token**. Save the username and password as the repository secrets `MAVEN_CENTRAL_USERNAME` and `MAVEN_CENTRAL_PASSWORD`.
4. Create a signing key, because Maven Central rejects unsigned artifacts:
   ```bash
   gpg --quick-gen-key "YorionEngine Releases <umesh@yorion.io>" rsa4096 sign 2y
   gpg --list-secret-keys --keyid-format short          # note the key id
   gpg --keyserver keyserver.ubuntu.com --send-keys <KEY_ID>
   gpg --armor --export-secret-keys <KEY_ID>             # whole block, including BEGIN/END lines
   ```
   Save the exported block as `MAVEN_SIGNING_KEY` and its passphrase as `MAVEN_SIGNING_KEY_PASSWORD`. Keep an offline backup of the key.
5. Put all four secrets in the `maven-central` environment rather than at repository level, so that only the publish job can read them.
6. Set `PUBLISH_MAVEN_CENTRAL=true`.

Releases are published automatically once they pass Central's validation. After the first upload, it can take up to about 30 minutes for a version to show up on Maven Central.

---

## Building packages locally

| Package | Command | Output |
| --- | --- | --- |
| Crate | `cd engine && cargo publish --dry-run` | `engine/target/package/` |
| npm | `./scripts/build-wasm.sh && ./scripts/package-npm.sh` | `dist/npm/` |
| Python wheel | `pip install maturin && cd python && maturin build` | `engine/target/wheels/` |
| Swift (macOS only) | `./scripts/generate-swift-bindings.sh && ./scripts/package-apple-spm.sh` | `dist/spm/` |
| Android AAR | `./scripts/build-android.sh publishToMavenLocal` (needs `ANDROID_HOME`, the NDK and `cargo-ndk`) | `~/.m2/repository/io/yorion/` |

## Things worth knowing

- **Python wheels use their own Cargo profile.** `[profile.python-release]` keeps symbols until UniFFI has generated the Python module. The normal release profile strips them, which silently produces an empty module. Don't pass `--release` to maturin, because that switches back to the stripped profile.
- **The Kotlin package name is `uniffi.yorion_engine`.** That is UniFFI's default, and the existing Flutter plugin's Android code imports it. Changing it to something like `io.yorion.engine` (`package_name` under `[bindings.kotlin]` in `engine/uniffi.toml`) is a breaking change, so do it before 1.0 if at all.
- **The npm package is about 1.5 MB** (5.9 MB unpacked) because it contains one WASM file per target (web, bundler, Node). An app loads only one, about 480 KB gzipped. Most of what remains is the IANA time-zone database pulled in by the `rrule` crate (used for `X-CALENDAR=AD` rules); it can only be trimmed by replacing `rrule`.
- **The tithi accuracy notice** ("verified through BS 2083") is in every package README. Update all of them when `TITHI_VERIFIED_THROUGH_BS` changes.

## Flutter / pub.dev (not automated yet)

The existing Flutter plugin lives outside this repository, in `apps/mobile/native/yorion_engine_flutter`, which is why `scripts/build-flutter.sh` copies binaries into it. Publishing it to pub.dev takes a few decisions that this change doesn't make:

1. **Move the plugin into this repo**, for example as `flutter/`, so every release builds it from the same source as the other packages.
2. **Choose how Dart calls Rust.** UniFFI has no official Dart backend. The options are:
   - Keep the current approach, where Dart calls the Swift and Kotlin bindings through platform channels. This is simplest, but iOS and Android only.
   - Use Dart **build hooks** (stable since late 2025), which compile the Rust code or download prebuilt binaries, together with generated `dart:ffi` bindings. This supports every Flutter platform.
   - Use `flutter_rust_bridge`, which needs a thin Rust API layer written for it.
3. **One-time pub.dev setup:**
   - Verify `yorion.io` as a publisher at <https://pub.dev/create-publisher> (DNS verification through Google Search Console).
   - Publish the first version by hand with `dart pub publish`.
   - Turn on **Automated publishing** in the package's admin tab: GitHub repository `Yorion-io/yorion_engine`, tag pattern `v{{version}}`, environment `pub-dev`.
   - Add a `pub-dev` job to `publish.yml`.
