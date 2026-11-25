# Agent Development Contract

This repository is CLI-first. The generated `appctl` CLI is the primary acceptance surface for application behavior.

For every API-facing change:

1. Update the Rust application under `src/`.
2. Update `openapi/openapi.yaml` in the same change. It is the API contract and CLI source of truth.
3. Run `make check` to regenerate, build, and test through `appctl`.
4. Commit the matching changes under `internal/generated/`, `skills/appctl/`, and `cmd/appctl/cli.yaml`.

Do not hand-edit generated files. Use direct HTTP only for transport-level checks that the generated CLI cannot express.

Before guessing a command or flag, use `appctl search "<intent>" --json`, then `appctl commands show <path...> --json`.

Keep the Rust application on Axum, Tokio, and Serde until a concrete requirement needs another dependency. Use Cargo for Rust commands. Do not add database, authentication, deployment, or compatibility scaffolding without an explicit requirement.

## Application versioning

Use [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html) for application releases:

- Use `X.Y.Z` for the version core, with nonnegative integers and no leading zeroes (clause 2).
- After publication, never change a version's contents, move its tag, or reuse its tag. Publish changes under a new version (clause 3).
- Use only `X.Y.Z-alpha.N`, `X.Y.Z-beta.N`, or `X.Y.Z-rc.N` for prereleases (clause 9). Start `N` at 1, with no leading zeroes.
- Compare versions by SemVer precedence (clause 11). Compare core numbers numerically. For the same core, `alpha < beta < rc < final`, and `rc.2 < rc.10`.
- Use `Cargo.toml` `[package].version` as the Rust application's version source. At the tagged commit, it must equal the version in `v<semver>`, without the tag prefix.
- If the Rust server and generated Go CLI release independently, give each artifact an independent version sequence and a fixed tag prefix. For example, use `v` for the server and `appctl-v` for the CLI. Each artifact's release tag and version metadata must represent the same logical version.

These rules apply to future releases. Do not rename, re-tag, or delete published tags.
