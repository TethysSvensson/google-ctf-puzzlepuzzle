#!/bin/sh
cargo run --release --bin dat2raw
git checkout shape_db.json
rm -f cached_groups.bin
