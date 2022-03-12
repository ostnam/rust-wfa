# rust-wfa

## Introduction
This project is an implementation of the wavefront alignment algorithm. This algorithm was first described in an article published in 2020: [Fast gap-affine pairwise alignment using the wavefront algorithm](https://doi.org/10.1093/bioinformatics/btaa777).

I have pasted part of the abstract below:
>In this article, we present the wavefront alignment algorithm (WFA), an exact gap-affine algorithm that takes advantage of homologous regions between the sequences to accelerate the alignment process. As opposed to traditional dynamic programming algorithms that run in quadratic time, the WFA runs in time O(ns), proportional to the read length n and the alignment score s, using O(s2) memory. Furthermore, our algorithm exhibits simple data dependencies that can be easily vectorized, even by the automatic features of modern compilers, for different architectures, without the need to adapt the code. We evaluate the performance of our algorithm, together with other state-of-the-art implementations. As a result, we demonstrate that the WFA runs 20–300× faster than other methods aligning short Illumina-like sequences, and 10–100× faster using long noisy reads like those produced by Oxford Nanopore Technologies."

The reference implementation of the algorithm, written in C by the authors of the article, is available at [https://github.com/smarco/WFA](github.com/smarco/WFA).

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
The number of characters of **Query** that are aligned, for a number of characters of **Text** aligned n and a diagonal d is equal to n + d. For instance, for the alignment of "CAT" and "CAC" *Matches<sub>i, j</sub> = 2*, since on diagonal 0, if the matching penalty/bonus is set to 0, we can match up to 2 characters ("CA").

We'll define insertions as aligning an extra character in *Query* (moving horizontally in the alignment matrix) and deletions as the opposite *Text* (moving vertically in the alignment matrix).

At a high-level, the wavefront alignment algorithm can be defined as such (with ``pens`` being a struct that holds the mismatch/gap penalties):

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

These relations are the same for WFA.
* For the *Deletes* matrix, the number of chars that can be matched at a score i and a diagonal j is the maximum of *Deletes<sub> i - gap extension penalty, j + 1 </sub>* and *Matches<sub>i - gap extension penalty - gap opening penalty, j + 1 </sub>* + 1. We add 1 due to the definition of the values stored in each cell.
* It's the same from the *Inserts* matrix, except that the values will come from the diagonal j - 1 (since insertions are a rightward movement in the SWG matrix). This time, we don't add 1 because the number stored in each cell is the number of characters of *Text* matched, and insertions are an extra character in *Query*.
* Matches is the maximum of *Deletes<sub>i, j</sub>*, *Inserts<sub>i, j</sub>* and *Matches<sub>i - mismatch pen, j</sub>*.

### Checking if the algorithm is over.
The final cell is at a specific diagonal: the diagonal whose number is the length of *Query* minus the length of *Text*.
At every cycle, we'll check if the number of characters matched at a diagonal is equal to the length of *Text*.

### Backtracking wavefronts to build an alignment
The backtracking algorithm isn't specified in the original article and I derived it myself since it's not very complicated. One important detail is not to forget that if we're on a match, we need to un-extend the wavefront.

```rust
while matches[current_score][current_diag] != score_at_the_originating_cell {
	// push the matching char to query_aligned and text_aligned
	matches[current_score][current_diag] -= 1;
}
```

## Rust implementation

## Validation
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

## Benchmarks
