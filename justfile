default:
  @just --list

# Cargo build everything.
build:
  cargo build --all-targets --all-features

# Cargo check everything.
check:
  cargo check --all-targets --all-features

# Lint everything.
lint:
  cargo +$(cat ./nightly-version) clippy --all-targets --all-features -- --deny warnings

# Check the formatting
format:
  cargo +$(cat ./nightly-version) fmt --all --check

# Run the formatter
fmt:
  cargo +$(cat ./nightly-version) fmt --all
