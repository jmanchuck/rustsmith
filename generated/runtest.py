import subprocess
import json
import math
import numpy as np
import pandas as pd
import os
import sys
from os import path

opt_levels = ["0", "3", "s"]

def mkdir_if_not_exist(path_dir):
    if not path.exists(path_dir):
        os.system(f"mkdir {path_dir}")

def delete_if_exists(target_path):
    if path.exists(target_path):
        os.system(f"rm -rf {target_path}")

def generate(seed):
    mkdir_if_not_exist("./src/bin/")
    os.system("cargo run --release -- -s " + str(seed))

def compile(seed):
    name = "seed_" + str(seed)
    mkdir_if_not_exist("executables")
    for opt_level in opt_levels:
        mkdir_if_not_exist(f"executables/{opt_level}")
        os.system(f"CARGO_PROFILE_RELEASE_OPT_LEVEL={opt_level}")
        os.system(f"cargo build --bin {name} --release --target-dir executables/{opt_level}")

    os.system("unset CARGO_PROFILE_RELEASE_OPT_LEVEL")

def run(seed):
    def checksum(result_dict):
        return sum(x for x in result_dict.values())
    filename = "seed_" + str(seed)
    checksums = []
    for opt_level in opt_levels:
        result = subprocess.Popen(f"timeout 5s ./executables/{opt_level}/release/{filename}", shell=True, stdout=subprocess.PIPE)
        result_str = result.stdout.read()
        if result_str:
            result_dict = json.loads(result_str)
            checksums.append((opt_level, checksum(result_dict)))
    
    return checksums
    

def test(count):
    differentials = []
    timeout_info = dict.fromkeys(opt_levels, 0)
    for i in range(count):
        generate(i)
        compile(i)
        result = run(i)
        if len(result) >= 2:
            if min(result, key=lambda x: x[1]) != max(result, key=lambda x: x[1]):
                differentials.append(i)
        for opt_level, _ in result:
            timeout_info[opt_level] += 1
    
    if len(differentials) > 0:
        print(f"Found differentials")
    else:
        print("No differentials found")

    with open("results", "w") as f:
        f.write(f"Total runs: {count}\n")
        f.write(f"Timeouts: {timeout_info}\n")
        f.write(f"Differentials: {differentials}")

def main():
    args = sys.argv
    if args[1] == "compile":
        compile(int(args[2]))

    elif args[1] == "generate":
        generate(int(args[2]))

    elif args[1] == "run":
        run(int(args[2]))

    elif args[1] == "test":
        test(int(args[2]))

    elif args[1] == "clean":
        delete_if_exists("./executables")
        delete_if_exists("./results")
        os.system("rm -rf ./src/bin/*.rs")

    elif args[1] == "format":
        os.system("rustfmt ./src/bin/*.rs")

if __name__ == "__main__":
    main()