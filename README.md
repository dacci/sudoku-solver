# Sudoku Solver

## Usage

```console
$ sudoku --help
Sudoku solver

Usage: sudoku <PATH>

Arguments:
  <PATH>  Path to the file that contains values of cells

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Example

```console
$ cat example.txt
207003004
000070100
308900060
005000003
070000020
600000500
060001705
009080000
400300609
$ sudoku example.txt
217863954
596472138
348915267
985126473
174539826
632748591
863291745
759684312
421357689

```
