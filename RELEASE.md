# Release Steps

1. Run `cargo workspace version` and choose one of `Patch`, `Minor`, or `Major`. This upgrades `Cargo.toml`
   and `Cargo.lock`, creates a Git tag, and pushes the changes to GitHub.
2. Wait until the GitHub Action produces the release.
3. Run `release-docker.sh` on `arm64`. It creates a Docker image for `arm64` and a manifest that references both `arm64`
   and `amd64` images. Currently, I have not found a way to automate creating `arm64` images using GitHub Actions
   because it gets stuck when using the approach from
   the [Docker Documentation](https://docs.docker.com/build/ci/github-actions/multi-platform/#distribute-build-across-multiple-runners).
