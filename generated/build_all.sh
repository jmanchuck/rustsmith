#!/bin/sh

generatedPath="../src/bin/*.rs"

cd executables 

rm -rf ./*/

mkdir opt3
mkdir optS 
mkdir opt0

for file in $generatedPath;
do
    filename=$(basename $file)
    cargo +nightly build --bins --profile opt3 -Z unstable-options --out-dir ./opt3
    cargo +nightly build --bins --profile optS -Z unstable-options --out-dir ./optS
    cargo +nightly build --bins --profile opt0 -Z unstable-options --out-dir ./opt0
done

find . -name "generated" -type f -delete