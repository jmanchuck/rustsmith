import json
import math
import numpy as np
import pandas as pd

f = open('./executables/output.json')
data = json.load(f)

cleaned = dict()

for opt_level in data:
    cleaned[opt_level] = dict()
    for seed in data[opt_level].keys():
        # NaN to indicate timed out
        if "timedout" in data[opt_level][seed]:
            continue
        struct_vals = list(data[opt_level][seed].values())
        checksum = sum(struct_vals)
        cleaned[opt_level][seed] = checksum

df = pd.DataFrame.from_dict(cleaned)

print(df)

print("Rows with unequal outputs:")

# Use NaN to indicate timed out, does not count for comparison, nanmin/max ignores nans
print(df[df.apply(lambda x: np.nanmin(x) != np.nanmax(x), 1)])