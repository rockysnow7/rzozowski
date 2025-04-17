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

def format_time(ns):
    """Format time in appropriate units based on magnitude."""
    if ns < 1000:
        return f"{ns:.2f} ns"
    elif ns < 1000000:
        return f"{ns/1000:.2f} μs"
    else:
        return f"{ns/1000000:.2f} ms"

# Calculate category averages across all patterns in a category
category_averages = {
    'parse_rzozowski': defaultdict(float),
    'parse_regex': defaultdict(float),
    'match_rzozowski_valid': defaultdict(float),
    'match_regex_valid': defaultdict(float),
    'match_rzozowski_invalid': defaultdict(float),
    'match_regex_invalid': defaultdict(float)
}

# Calculate averages for parsing
for category in ["simple", "intermediate", "complex"]:
    if category in parse_summary.get("rzozowski", {}):
        rz_times = parse_summary["rzozowski"][category]
        category_averages['parse_rzozowski'][category] = sum(rz_times) / len(rz_times)
    
    if category in parse_summary.get("regex", {}):
        regex_times = parse_summary["regex"][category]
        category_averages['parse_regex'][category] = sum(regex_times) / len(regex_times)

# Calculate averages for matching
for category in ["simple", "intermediate", "complex"]:
    if category in match_summary.get("rzozowski", {}).get("valid", {}):
        rz_valid_times = match_summary["rzozowski"]["valid"][category]
        category_averages['match_rzozowski_valid'][category] = sum(rz_valid_times) / len(rz_valid_times)
    
    if category in match_summary.get("regex", {}).get("valid", {}):
        regex_valid_times = match_summary["regex"]["valid"][category]
        category_averages['match_regex_valid'][category] = sum(regex_valid_times) / len(regex_valid_times)
    
    if category in match_summary.get("rzozowski", {}).get("invalid", {}):
        rz_invalid_times = match_summary["rzozowski"]["invalid"][category]
        category_averages['match_rzozowski_invalid'][category] = sum(rz_invalid_times) / len(rz_invalid_times)
    
    if category in match_summary.get("regex", {}).get("invalid", {}):
        regex_invalid_times = match_summary["regex"]["invalid"][category]
        category_averages['match_regex_invalid'][category] = sum(regex_invalid_times) / len(regex_invalid_times)

# Define categories for each section
parsing_categories = ["simple", "intermediate", "complex"]
matching_categories = ["simple", "intermediate", "complex"]

# Generate formatted output
print("# Benchmark Results")
print()
print("## Regex Parsing Performance")
print()
print("| Category | rzozowski | regex | Ratio (rzozowski/regex) |")
print("|----------|-----------|-------|--------------------------|")
for category in parsing_categories:
    if category in category_averages['parse_rzozowski'] and category in category_averages['parse_regex']:
        rz_time = category_averages['parse_rzozowski'][category]
        regex_time = category_averages['parse_regex'][category]
        ratio = rz_time / regex_time
        print(f"| {category.capitalize()} | {format_time(rz_time)} | {format_time(regex_time)} | {ratio:.2f} |")

print()
print("## Regex Matching Performance (valid inputs)")
print()
print("| Category | rzozowski | regex | Ratio (rzozowski/regex) |")
print("|----------|-----------|-------|--------------------------|")
for category in matching_categories:
    if category in category_averages['match_rzozowski_valid'] and category in category_averages['match_regex_valid']:
        rz_time = category_averages['match_rzozowski_valid'][category]
        regex_time = category_averages['match_regex_valid'][category]
        ratio = rz_time / regex_time
        print(f"| {category.capitalize()} | {format_time(rz_time)} | {format_time(regex_time)} | {ratio:.2f} |")

print()
print("## Regex Matching Performance (invalid inputs)")
print()
print("| Category | rzozowski | regex | Ratio (rzozowski/regex) |")
print("|----------|-----------|-------|--------------------------|")
for category in matching_categories:
    if category in category_averages['match_rzozowski_invalid'] and category in category_averages['match_regex_invalid']:
        rz_time = category_averages['match_rzozowski_invalid'][category]
        regex_time = category_averages['match_regex_invalid'][category]
        ratio = rz_time / regex_time
        print(f"| {category.capitalize()} | {format_time(rz_time)} | {format_time(regex_time)} | {ratio:.2f} |")
