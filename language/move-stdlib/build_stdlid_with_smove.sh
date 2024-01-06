#!/usr/bin/env bash

set -e

# Build the move-stdlib bundle
# Comment: source files could eventually be moved to another repo.
smove bundle

# Build the substrate-stdlib bundle
pushd .
rm -rf substrate-stdlib
git clone https://github.com/eigerco/substrate-stdlib.git
cd substrate-stdlib
smove bundle
popd
