# TODO

A command line todo list

## Installation

Build and install from source using cargo
```
cargo install --path .
```

## Usage

There are four modes supported
```bash
TODO               # see all items in to-do list
TODO message       # add an item to the list
TODO --complete N  # mark an item in the list as completed
TODO --delete N    # delete an item from the list
```

Example usage
```bash
TODO Update README.md
TODO
# 1 [ ] Update README.md (2025-11-07 22:18:21)
vim README.md
TODO --complete 1
TODO
# 1 [x] Update README.md (2025-11-07 22:18:21)
```
