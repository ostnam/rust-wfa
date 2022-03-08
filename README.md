# rust-wfa

## Introduction
This project is an implementation of the wavefront alignment algorithm. This algorithm was first described in an article published in 2020: [Fast gap-affine pairwise alignment using the wavefront algorithm](https://doi.org/10.1093/bioinformatics/btaa777).

I have pasted part of the abstract below:
>In this article, we present the wavefront alignment algorithm (WFA), an exact gap-affine algorithm that takes advantage of homologous regions between the sequences to accelerate the alignment process. As opposed to traditional dynamic programming algorithms that run in quadratic time, the WFA runs in time O(ns), proportional to the read length n and the alignment score s, using O(s2) memory. Furthermore, our algorithm exhibits simple data dependencies that can be easily vectorized, even by the automatic features of modern compilers, for different architectures, without the need to adapt the code. We evaluate the performance of our algorithm, together with other state-of-the-art implementations. As a result, we demonstrate that the WFA runs 20–300× faster than other methods aligning short Illumina-like sequences, and 10–100× faster using long noisy reads like those produced by Oxford Nanopore Technologies."

## Wavefronts explained
### Wavefronts basics
This algorithm computes the optimal alignment between a pair of strings *Query* and *Text* using 2-dimensional matrices: *Insert<sub>i, j</sub>*, *Matches<sub>i, j</sub>* and *Deletes<sub>i, j</sub>*. 

Unlike aligmnent algorithms based on dynamic programming (for instance the Smith-Waterman-Gotoh algorithm), *i* and *j* do **not** correspond to a position in *Query* or *Text*.

*i* corresponds to a **score**.

*j* corresponds to a diagonal in the dynamic programming matrix that would be used to align *Query* and *Text*. The following figure shows how these diagonals are laid out.

|    | " " | C  | A  | T |
|----|----|----|----|---|
| " " | 0  | 1  | 2  | 3 |
| C  | -1 | 0  | 1  | 2 |
| A  | -2 | -1 | 0  | 1 |
| C  | -3 | -2 | -1 | 0 |

The value that is stored in each of the matrix is the **number of characters of** ***Text***  that can be aligned, for the score *i*, at the diagonal *j*.

### The wavefront recurrence relation

### Matches extension

### The final alignment function

### Backtracking wavefronts to build an alignment

## Rust implementation

## Validation

## Benchmarks

## 
