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
            cleaned[opt_level][seed] = None
            continue
        struct_vals = list(data[opt_level][seed].values())
        checksum = sum(struct_vals)
        cleaned[opt_level][seed] = checksum

df = pd.DataFrame.from_dict(cleaned)

def can_differential(row):
    return row.count() >= 2

def has_differential(row):
    if can_differential(row):
        return np.nanmin(row) != np.nanmax(row)
    else:
        return False

print("Number of timeouts:")
print(df.isna().sum(axis=0))

unequal_rows = df[df.apply(has_differential, 1)]

if len(unequal_rows) == 0:
    print("No differential found")
else:
    # Use NaN to indicate timed out, does not count for comparison, nanmin/max ignores nans
    print("Rows with unequal outputs:")
    print(unequal_rows)

