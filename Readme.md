# fdu -- a tool to flamegraph disk usage

This tool recursively walks over directories and prints disk usage.

```
Usage: `fdu [-bdfFhlr] <path>...`

Options:
    -b, --buffer        buffer output, speeds up printing
    -d, --duplicates    include multiple instances of the same file
    -f, --flamegraph    output disk usage in flamegraph-friendly format
    -F, --full-name     do not remove root from file paths
    -h, --help          print usage
    -l, --list          list each file (automatic with -f)
    -r, --size-first    print file size first
```
