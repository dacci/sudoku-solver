# Sudoku Solver

## Usage

```console
sudoku 0.1.0
Sudoku solver

USAGE:
    sudoku <PATH>

ARGS:
    <PATH>    Path to the file that contains values of cells

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
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
