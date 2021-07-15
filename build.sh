#!/bin/sh

cd smith
cargo build
cd ..

cd runtime
cargo build
cd ..

cd generated
cargo +nightly build
cd ..