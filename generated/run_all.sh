#!/bin/sh

cd executables
rm output.json 

echo "{" >> output.json 

for dir in ./*/;
do 
    cd $dir 
    echo "Running binaries from $dir"
    echo "\"$(basename $dir)\": {" >> ../output.json
    for file in ./release/*;
    do 
        if [[ -f $file && -x $file ]]; then
            output=$(timeout 1s ./$file)
            if test -z "$output"; then 
                echo "\"$(basename $file)\": {\"timedout\": -1}" >> ../output.json
            else
                echo "\"$(basename $file)\": $output" >> ../output.json
            fi
            echo "," >> ../output.json
        fi
    done
    sed -i '' -e '$ d' ../output.json
    echo "}" >> ../output.json
    echo "," >> ../output.json
    cd ..
done
sed -i '' -e '$ d' ./output.json

echo "}" >> output.json