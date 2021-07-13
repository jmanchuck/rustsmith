#!/bin/sh

cd executables
rm output.json 

echo "{" >> output.json 

for dir in ./*/;
do 
    cd $dir 
    echo "\"$(basename $dir)\": {" >> ../output.json
    for file in ./*;
    do 
        output=$(./$file)
        echo "\"$(basename $file)\": $output," >> ../output.json
    done
    echo "}," >> ../output.json
    cd ..
done

echo "}" >> output.json