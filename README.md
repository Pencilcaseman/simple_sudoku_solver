# Simple Sudoku Solver

This is a simple sudoku solver written in Rust. It uses the wavefunction
collapse algorithm to solve the puzzle as far as possible, before backtracking
to "brute force" the rest of it.

Maybe one day I'll come up with a solution that actually calculates the value
for each cell. This is good enough for now, though, and is surprisingly fast.
