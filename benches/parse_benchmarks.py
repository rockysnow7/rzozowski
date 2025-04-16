#!/usr/bin/env python3
import json
import os
from collections import defaultdict

# Define the categories for grouping
CATEGORIES = {
    "simple": ["concatenation", "alternation", "star", "plus", "question", "class"],
    "intermediate": ["special_char_sequences", "count", "nested_star"],
    "complex": ["deep_nesting", "complex_count", "complex_class", "exponential_plus", "email"]
}

# Function to find category for a pattern
def get_category(pattern):
    for category, patterns in CATEGORIES.items():
        if pattern in patterns:
            return category
    return "unknown"

# Initialize data structures
parse_results = defaultdict(lambda: defaultdict(list))
match_results = defaultdict(lambda: defaultdict(lambda: defaultdict(list)))

# Process all estimates.json files
for root, dirs, files in os.walk("target/criterion"):
    if "estimates.json" in files and "new" in root:
        file_path = os.path.join(root, "estimates.json")
        
        # Extract components from path
        components = root.split(os.sep)
        benchmark_type = components[2]  # regex_parse or regex_matches
        
        if benchmark_type == "regex_parse":
            implementation = components[3]  # rzozowski or regex
            pattern = components[4]  # the pattern name
            
            # Read the benchmark result
            with open(file_path, 'r') as f:
                data = json.load(f)
                median_ns = data["median"]["point_estimate"]
                parse_results[pattern][implementation].append(median_ns)
                
        elif benchmark_type == "regex_matches":
            match_type = components[3]  # rzozowski-valid, rzozowski-invalid, regex-valid, regex-invalid
            pattern = components[4]  # the pattern name
            
            # Split implementation and validity
            impl_valid = match_type.split('-')
            implementation = impl_valid[0]
            validity = impl_valid[1] if len(impl_valid) > 1 else "unknown"
            
            # Read the benchmark result
            with open(file_path, 'r') as f:
                data = json.load(f)
                median_ns = data["median"]["point_estimate"]
                match_results[pattern][implementation][validity].append(median_ns)

# Calculate averages for parse results
parse_summary = defaultdict(dict)
for pattern, impls in parse_results.items():
    category = get_category(pattern)
    for impl, times in impls.items():
        if category not in parse_summary[impl]:
            parse_summary[impl][category] = []
        parse_summary[impl][category].extend(times)

# Calculate averages for match results
match_summary = defaultdict(lambda: defaultdict(dict))
for pattern, impls in match_results.items():
    category = get_category(pattern)
    for impl, validities in impls.items():
        for validity, times in validities.items():
            if category not in match_summary[impl][validity]:
                match_summary[impl][validity][category] = []
            match_summary[impl][validity][category].extend(times)

# Print markdown tables
def print_table(title, data):
    print(f"## {title}")
    print("\n| Category | rzozowski (ns) | regex (ns) | Ratio (rzozowski/regex) |")
    print("|----------|----------------|------------|--------------------------|")
    
    for category in ["simple", "intermediate", "complex"]:
        if "rzozowski" in data and category in data["rzozowski"] and "regex" in data and category in data["regex"]:
            rzo_avg = sum(data["rzozowski"][category]) / len(data["rzozowski"][category])
            reg_avg = sum(data["regex"][category]) / len(data["regex"][category])
            ratio = rzo_avg / reg_avg
            print(f"| {category.capitalize()} | {rzo_avg:.2f} | {reg_avg:.2f} | {ratio:.2f} |")
    print()

# Generate output for README
print("# Benchmark Results\n")

print_table("Regex Parsing Performance", parse_summary)

for validity in ["valid", "invalid"]:
    rzo_data = {category: times for category, times in match_summary["rzozowski"][validity].items()}
    reg_data = {category: times for category, times in match_summary["regex"][validity].items()}
    combined_data = {"rzozowski": rzo_data, "regex": reg_data}
    print_table(f"Regex Matching Performance ({validity} inputs)", combined_data)
