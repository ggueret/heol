# Contributing

## Development

```bash
make check   # fmt + clippy + tests
make build   # release build
```

## Commits

This project uses [Conventional Commits](https://www.conventionalcommits.org/).
Commit messages must be ASCII-only.

```
type(scope): description      <- max 50 chars total

[optional body]               <- wrap at 72 chars
```

## Releasing

The changelog is generated from conventional commits using [git-cliff](https://git-cliff.org/).

```bash
# Preview the changelog
git cliff

# Update CHANGELOG.md
git cliff -o CHANGELOG.md

# Tag and push to trigger the release workflow
git tag vX.Y.Z
git push origin main --tags
```

The CI builds binaries for all targets, generates release notes with git-cliff, and publishes a GitHub release with checksums.
