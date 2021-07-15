#!/bin/sh

cd executables
rm output.json 

echo "{" >> output.json 

for dir in ./*/;
do 
    cd $dir 
    echo "Running binaries from $dir"
    echo "\"$(basename $dir)\": {" >> ../output.json
    for file in ./*;
    do 
        # echo "Checking $file"
        if [[ -f $file && -x $file ]]; then
            # echo "Running $file"
            output=$(./$file)
            echo "\"$(basename $file)\": $output," >> ../output.json
        fi
    done
    echo "}," >> ../output.json
    cd ..
done

echo "}" >> output.json