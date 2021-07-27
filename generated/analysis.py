import json
import numpy as np
import pandas as pd

f = open('./executables/output.json')
data = json.load(f)

cleaned = dict()

for opt_level in data:
    cleaned[opt_level] = dict()
    for seed in data[opt_level].keys():
        struct_vals = list(data[opt_level][seed].values())
        checksum = sum(struct_vals)
        cleaned[opt_level][seed] = checksum

df = pd.DataFrame.from_dict(cleaned)

print(df)

print("Rows with unequal outputs:")

print(df[df.apply(lambda x: min(x) != max(x), 1)])
