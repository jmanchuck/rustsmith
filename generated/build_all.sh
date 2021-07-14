#!/bin/sh

generatedPath="../src/bin/*.rs"

cd executables 

rm -rf ./*/

for file in $generatedPath;
do
    filename=$(basename $file)
    cargo build --bins --release --target-dir .
    cargo build --bins --target-dir .
    # cargo +nightly build --bins --profile optS --target-dir . -Z unstable-options

done

find . -name "generated" -type f -delete