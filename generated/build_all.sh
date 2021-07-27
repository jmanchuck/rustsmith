#!/bin/sh

generatedPath="../src/bin/*.rs"

declare -a opt_levels=("0" "3" "s")

if [ ! -d "./executables" ]
then
    mkdir executables 
fi

cd executables 

rm -rf ./*/
for opt_level in "${opt_levels[@]}";
do
    i=0
    echo $opt_level
    mkdir $opt_level
    export CARGO_PROFILE_RELEASE_OPT_LEVEL=$opt_level
    for file in $generatedPath;
        do
            filename=$(basename $file)
            cargo build --bins --release --target-dir ./"$opt_level"/
            # cargo build --bins --target-dir .
            # cargo +nightly build --bins --profile optS --target-dir . -Z unstable-options
        done
done

unset "CARGO_PROFILE_RELEASE_OPT_LEVEL"

find . -name "generated" -type f -delete