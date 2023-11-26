#!/usr/bin/env bash

# `cargo publish` ignores directories containing a Cargo.toml
git submodule foreach "rm -f Cargo.toml"

# Record the version, so the build script has access to it
git submodule foreach "git rev-parse HEAD > pepegsitter-version"

# Run the publish, user needs to pass --allow-dirty, just to make sure ...
cargo publish $@

# Reset the changes just made.
git submodule foreach "git reset --hard HEAD && git clean -f"
