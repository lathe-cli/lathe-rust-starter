# CLI-first Rust app

A minimal Rust application where the CLI is ready on day one. Change the application, update OpenAPI, and test through the generated `appctl` CLI.

Requires Rust 1.96+ and Go 1.26+.

```sh
make check
cargo run
```

In another terminal:

```sh
./bin/appctl search "create task" --json
./bin/appctl commands show tasks create --json
./bin/appctl tasks create --set title="Ship from the CLI" -o json
```

`make check` regenerates the CLI from `openapi/openapi.yaml`, builds it, and runs acceptance tests through CLI commands. `make test` only builds and tests the checked-in generated CLI.

Generated output under `internal/generated/` and `skills/appctl/` is committed with API changes. The pipeline uses [Lathe](https://github.com/lathe-cli/lathe), pinned by version.

## Application versioning

Use [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html) for application releases:

- Use `X.Y.Z` for the version core, with nonnegative integers and no leading zeroes (clause 2).
- After publication, never change a version's contents, move its tag, or reuse its tag. Publish changes under a new version (clause 3).
- Use only `X.Y.Z-alpha.N`, `X.Y.Z-beta.N`, or `X.Y.Z-rc.N` for prereleases (clause 9). Start `N` at 1, with no leading zeroes.
- Compare versions by SemVer precedence (clause 11). Compare core numbers numerically. For the same core, `alpha < beta < rc < final`, and `rc.2 < rc.10`.
- Use `Cargo.toml` `[package].version` as the Rust application's version source. At the tagged commit, it must equal the version in `v<semver>`, without the tag prefix.
- If the Rust server and generated Go CLI release independently, give each artifact an independent version sequence and a fixed tag prefix. For example, use `v` for the server and `appctl-v` for the CLI. Each artifact's release tag and version metadata must represent the same logical version.

These rules apply to future releases. Do not rename, re-tag, or delete published tags.
