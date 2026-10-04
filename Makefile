.PHONY: cli-sync cli-build test check

cli-sync:
	cp cli.yaml cmd/appctl/cli.yaml
	go tool lathe bootstrap
	go mod tidy

cli-build:
	go build -o bin/appctl ./cmd/appctl

test: cli-build
	cargo test --locked

check: cli-sync cli-build
	cargo fmt --check
	cargo clippy --locked --all-targets -- -D warnings
	cargo test --locked
	go vet ./...

