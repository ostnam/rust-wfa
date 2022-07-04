# rust-wfa

## Introduction
This project is an implementation of the wavefront alignment algorithm. This algorithm was first described in an article published in 2020: [Fast gap-affine pairwise alignment using the wavefront algorithm](https://doi.org/10.1093/bioinformatics/btaa777).

I have pasted part of the abstract below:
>In this article, we present the wavefront alignment algorithm (WFA), an exact gap-affine algorithm that takes advantage of homologous regions between the sequences to accelerate the alignment process. As opposed to traditional dynamic programming algorithms that run in quadratic time, the WFA runs in time O(ns), proportional to the read length n and the alignment score s, using O(s2) memory. Furthermore, our algorithm exhibits simple data dependencies that can be easily vectorized, even by the automatic features of modern compilers, for different architectures, without the need to adapt the code. We evaluate the performance of our algorithm, together with other state-of-the-art implementations. As a result, we demonstrate that the WFA runs 20–300× faster than other methods aligning short Illumina-like sequences, and 10–100× faster using long noisy reads like those produced by Oxford Nanopore Technologies."

The reference implementation of the algorithm, written in C by the authors of the article, is available at [github.com/smarco/WFA2-lib](https://github.com/smarco/WFA2-lib).

## Wavefronts explained
### Wavefronts basics
This explanation assumes that the reader is familiar with the [Smith-Waterman-Gotoh sequence alignment algorithm,](https://en.wikipedia.org/wiki/Smith%E2%80%93Waterman_algorithm) and its gap-affine version.

The wavefront alignment algorithm computes the optimal alignment between a pair of strings *Query* and *Text* using three 2-dimensional matrices: *Insert<sub>i, j</sub>*, *Matches<sub>i, j</sub>* and *Deletes<sub>i, j</sub>*.

Unlike SWG alignment, *i* and *j* do **not** correspond to a position in *Query* or *Text*.

* *i* corresponds to a **score**.

* *j* corresponds to a diagonal in the dynamic programming matrix that would be used to align *Query* and *Text*. The following figure shows how these diagonals are laid out.

|    | " " | C  | A  | T |
|----|----|----|----|---|
| " " | 0  | 1  | 2  | 3 |
| C  | -1 | 0  | 1  | 2 |
| A  | -2 | -1 | 0  | 1 |
| C  | -3 | -2 | -1 | 0 |

The value that is stored in each of the wavefront matrices cell is the **number of characters of** ***Text*** that can be aligned, for the score *i*, at the diagonal *j*.

If we align n character of **Text**, the number of characters of **Query** that are aligned is equal to n + the number of the current diagonal.
For instance, for the alignment of "CAT" and "CAC" *Matches<sub>i, j</sub> = 2*, since on diagonal 0, if the matching penalty/bonus is set to 0, we can match up to 2 characters ("CA").

We'll define the direction of insertions as aligning an extra character in *Query* (moving horizontally in the alignment matrix) and deletions as the opposite *Text* (moving vertically in the alignment matrix).

At the highest level, the wavefront alignment algorithm can be defined as such (with ``pens`` being a struct that holds the mismatch/gap penalties):

```rust
    let mut current_front = new_wavefront_state(query, text, pens);
    loop {
        current_front.extend();
        if current_front.is_finished() {
            break;
        }
        current_front.increment_score();
        current_front.next();
    }
```

### Matches extension
One key aspect of wavefront alignment is that the matching penalty is set to 0.
Recall the definition of the value stored in each matrix: 
> The value that is stored in each of the matrices cell is the **number of characters of** ***Text*** that can be aligned, for the score *i*, at the diagonal *j*.

For a given position the score can be extended as long as the character of *Text* and *Query*, in the diagonal of the SWG matrix match. We can thus obtain the wavefront extension function:
```rust
for diag in current_diagonals {
	let mut x = matches[current_score][diag];
	while text[x] == query[x + diag] {
		x += 1;
		matches[current_score][diag] = x;
	}
}
```


### The wavefront recurrence relation
Smith-Waterman-Gotoh defines the following recurrence relation to build the alignment matrices:

* *Deletes<sub>i, j</sub> = min( Deletes<sub>i - 1, j</sub> + gap extension penalty, Matches<sub>i - 1, j</sub> + gap extension penalty + gap opening penalty )*
* *Inserts<sub>i, j</sub> = min( Inserts<sub>i, j - 1 </sub> + gap extension penalty, Matches<sub>i, j - 1 </sub> + gap extension penalty + gap opening penalty )*
* *Matches<sub>i, j</sub> = min( Matches<sub>i, j</sub> + the penalty of matching/mismatching Text<sub>i</sub> to Query<sub>j</sub>, Inserts<sub>i, j</sub>, Deletes<sub>i, j</sub> )*

The WFA\_next function is based on these relations. The WFA\_extend function is equivalent to going down a diagonal of the DP matrix while *Text<sub>i</sub>== Query<sub>j</sub>*. Once a cell where this equation doens't hold is reached, WFA\_next is equivalent to computing the DP cells to the left and right of that cell.

Rephrasing the SWG recurrence relations for wavefronts gives these relations:
* For the *Deletes* matrix, the number of chars that can be matched at a score i and a diagonal j is the maximum of *Deletes<sub> i - gap extension penalty, j + 1 </sub>* and *Matches<sub>i - gap extension penalty - gap opening penalty, j + 1 </sub>* + 1. The '+ 1' is due to the definition of the values stored in each cell.
* It's the same from the *Inserts* matrix, except that the values will come from the diagonal j - 1 (since insertions are a rightward movement in the SWG matrix). This time, we don't add 1 because the number stored in each cell is the number of characters of *Text* matched, and insertions are an extra character in *Query*.
* Matches is the maximum of *Deletes<sub>i, j</sub>*, *Inserts<sub>i, j</sub>* and *Matches<sub>i - mismatch pen, j</sub>*.

### Checking if the algorithm is over.
The final cell is at a specific diagonal: the diagonal whose number is the length of *Query* minus the length of *Text*.
At every cycle, we'll check if the number of characters matched at a diagonal is equal to the length of *Text*.

### Backtracking wavefronts to build an alignment
The backtracking algorithm isn't specified in the original article and I derived it myself since it's not very complicated. One important detail is not to forget that if we're on a match, we need to un-extend the wavefront.

## Rust implementation

### Validation of my implementation

#### Verifying that the WFA algorithm gives the same score as SWG alignment.
I have implemented a naive, unoptimized version of the SWG alignment in reference.rs.
I have written validate.rs (a binary target that gets compiled to a standalone executable binary file).
```
./target/release/validate -h
rust_wfa 0.1.0

USAGE:
    validate [OPTIONS] --min-length <MIN_LENGTH> --max-length <MAX_LENGTH> --min-error <MIN_ERROR> --max-error <MAX_ERROR>

OPTIONS:
    -h, --help                       Print help information
        --max-error <MAX_ERROR>      
        --max-length <MAX_LENGTH>    
        --min-error <MIN_ERROR>      
        --min-length <MIN_LENGTH>    
    -p, --parallel                   
    -V, --version  
```
That program generates random strings of a length in the interval specified by the user and a second, mutated version of that string that differs by the error rate (in percent) interval given. It then aligns the 2 strings using both the WFA and SWG algorithm, and checks that their score is the same (the alignment itself is not compared since there can be multiple alignment for an optimal alignment score).
It can also run in parallel, doing this process concurrently, with a different text/query pair of strings over each detected cpu core.

After using this executable to fix the remaining bugs in my algorithm, I have now been able to compare the alignments of hundred thousands of strings without a difference in the alignment score between both algorithms, which has convinced me of the soundness of my implementation.

#### Verifying that the alignment and its score matches.
In a similar manner, I wrote validate\_score\_matches\_alignment. This program gets compiled to a binary with the same name, that can be ran to generate pairs of strings, align them, and verify that the alignment matches its score: the score is recomputed from the alignment and then compared.

```
./target/release/validate_score_matches_alignment -h
rust_wfa 0.1.0
USAGE:
    validate_score_matches_alignment [OPTIONS] --min-length <MIN_LENGTH> --max-length <MAX_LENGTH> --min-error <MIN_ERROR> --max-error <MAX_ERROR>

OPTIONS:
    -h, --help                       Print help information
        --max-error <MAX_ERROR>      
        --max-length <MAX_LENGTH>    
        --min-error <MIN_ERROR>      
        --min-length <MIN_LENGTH>    
    -p, --parallel                   
    -V, --version                    Print version information
```


This program allowed me to identify another bug (a single incorrect variable) that produced incorrect alignments, with a correct score (and thus weren't detected by the previous validation program).
I can now validate thousands of alignments, without producing bugs.

### Benchmarks

I first ran the benchmarks on my naive implementation. The results are in the table below.
n = length of the sequences, d = rate at which the elements differ between each sequence.
|          | n = 100, d = 1% | n = 100, d = 10% | n = 100, d = 30% | n = 1k, d = 1% | n = 1k, d = 10% | n = 1k, d = 30% | n = 10k, d = 1% | n = 10k, d = 10% | n = 10k, d = 30% |
|:--------:|:-----------:|:-----------:|:------------:|:----------:|:----------:|:------------:|:------------:|:------------:|:-------------:|
| rust-wfa | 41 µs  | 131 µs | 291 µs |  1.77 ms  | 16 ms    |   33 ms |  202.6 ms | 1.59 s   | 3.3 s    |
| WFA2     |  5 µs  |  25 µs |  53 µs |     42 µs |   665 µs |    2 ms |    1 ms   |    37 ms |   140 ms |
| WFA2 SWG | 87 µs  |  90 µs |  95 µs | 11 ms     | 11 ms    |   11 ms | 1 s       | 1 s      | 1 s      |

As expected, SWG alignment doesn't depend on the error rate. My naive implementation is much slower than the original implementation, and it is only faster than SWG alignment for highly similar sequences.

This was an initial, naive implementation of the WFA algorithm. For instance, I stored the wavefronts as Vec<Vec<i32>>, which isn't very efficient.

In the next version, I rewrote my implementation to use a more efficient 1D Vec<i32>.
|          | n = 100, d = 1% | n = 100, d = 10% | n = 100, d = 30% | n = 1k, d = 1% | n = 1k, d = 10% | n = 1k, d = 30% | n = 10k, d = 1% | n = 10k, d = 10% | n = 10k, d = 30% |
|:--------:|:-----------:|:-----------:|:------------:|:----------:|:----------:|:------------:|:------------:|:------------:|:-------------:|
| rust-wfa | 23 µs       | 82 µs | 227 µs | 164 µs  | 4.6 ms    |   14.1 ms |  7.7 ms | 244 ms | 1.1 s |
| WFA2     |  28 µs  |  147 µs | 332 µs | 220 µs |   1.5 ms |    3.6 ms |    2.3 ms   | 25 ms | 87 ms |
| WFA2 SWG | 82 µs  |  102 µs |  103 µs | 11 ms     | 10 ms    |   7 ms | 1 s       | 1 s      | 1 s      |

The runtime was reduced by ~90%. My implementation is now competitive with the reference one, and efficient SWG.
