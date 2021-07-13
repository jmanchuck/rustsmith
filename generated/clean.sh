#!/bin/sh

while true; do
    read -p "Do you want to delete all generated Rust programs (y/n)? " yn
    case $yn in
        [Yy]* ) rm ./src/bin/*.rs ; rm -rf ./executables/*/; rm ./executables/output.json ; break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done