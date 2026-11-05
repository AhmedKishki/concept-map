# concept-graph

`concept-graph` builds the concept graph of a text from its passages. A concept is a phrase from the text, such as "march hare" or "transcendental realism", that marks one of its topics. Two concepts are linked when they stand in the same passages more often than chance predicts, and linked concepts fall into communities. Every concept and every link carries the ids of its passages, so each one can be checked against the text.

`build` makes the graph in five steps:

1. Candidate phrases: the runs of one to three content words between stopwords and punctuation, as RAKE cuts them, that stand in at least 3 passages and in at most 30% of them.
2. Topics: the passages sorted into round(√n) topics by non-negative matrix factorization (NMF) of their TF-IDF weights.
3. Concepts: the 60 candidates with the most mutual information with the topics.
4. Links: for each concept, up to three partners with the highest normalized pointwise mutual information (NPMI) above 0.1, among the concepts that share at least 2 passages with it.
5. Communities: the Louvain method on the links, weighted by their NPMI.

[docs/concept-graph.md](docs/concept-graph.md) explains each step, its maths and its choices, with worked examples.

## Adding it

The crate is not on crates.io, so add it as a git dependency:

```toml
[dependencies]
concept-graph = { git = "https://github.com/AhmedKishki/concept-graph" }
```

It needs Rust 1.92 or later and depends on `frostem` and `serde`.

## Usage

```rust
fn main() -> Result<(), concept_graph::Error> {
    // 24 passages in reading order. Each scene recurs in 6 of them, so its
    // phrases are frequent enough to be concepts and rare enough to tell
    // the scenes apart.
    let scenes = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ];
    let texts: Vec<&str> = (0..24).map(|i| scenes[i % 4]).collect();

    let graph = concept_graph::build(&texts)?;
    for community in &graph.communities {
        let labels: Vec<&str> = community
            .concepts
            .iter()
            .map(|&id| graph.concepts[id as usize].label.as_str())
            .collect();
        println!("{}", labels.join(", "));
    }
    let link = &graph.links[0];
    println!(
        "{} and {}: NPMI {}, passages {:?}",
        graph.concepts[link.a as usize].label,
        graph.concepts[link.b as usize].label,
        link.npmi,
        link.passages
    );
    Ok(())
}
```

It prints

```text
crying, gryphon danced, lobster quadrille, mock turtle
dormouse, hatter, march hare, tea
hearts, knave, queen, trial
alice, long hall, rabbit hole
crying and gryphon danced: NPMI 1, passages [0, 4, 8, 12, 16, 20]
```

## Input

`build(texts: &[&str])` takes the passages of one text, in reading order.

- A passage is the unit of co-occurrence: two concepts occur together when both stand in one passage, wherever they are inside it. The constants below were chosen for passages of whole paragraphs of at most 220 words.
- There must be at least 20 passages.
- A passage's id in the output is its index in `texts`. Concepts are numbered in the order of their first passage, and ties between communities go to the one the text mentions first, so the order of `texts` matters.
- The texts need no preparation: `build` lowercases, splits and stems the words itself. Whitespace, line breaks included, only separates words, so a heading or list item that ends without punctuation runs into the words after it in the same passage.
- Everything in the passages counts. Notes, a contents page, an index, running heads and licence text give concepts like the body of the text, so remove what should not count before calling `build`.

## Output

`build` returns a `GraphResult`. It and the types it holds derive serde's `Serialize` and `Deserialize`.

| `GraphResult` field | What it holds |
|---|---|
| `passages` | n, the number of passages |
| `topics` | k, the number of topics the concepts were chosen with |
| `concepts` | at most 60 `Concept`s, in the order of their first passage |
| `links` | the `Link`s, sorted by `a` and then `b` |
| `communities` | the `Community`s, largest first |

| `Concept` field | What it holds |
|---|---|
| `id` | its index in `concepts` |
| `label` | the lowercased form the text most often writes it in |
| `passage_count` | how many passages it stands in as a phrase of its own, cut off from other words by stopwords or punctuation |
| `passages` | those passages' ids, ascending |
| `community` | the index of its community in `communities` |

| `Link` field | What it holds |
|---|---|
| `a`, `b` | the ids of its two concepts, `a` the lower |
| `npmi` | NPMI rounded to three decimals: 1 when the two only ever stand together, 0 when they share passages as often as chance predicts |
| `passage_count` | how many passages both concepts stand in |
| `passages` | those passages' ids, ascending |

| `Community` field | What it holds |
|---|---|
| `id` | its index in `communities` |
| `concepts` | its concepts' ids, ascending |

A concept stands in a passage only as a run of its own: "turtle" inside "mock turtle" does not count for a concept "turtle". A text in which no phrase stands in at least 3 passages and in at most 30% of them has no concepts, so `build` returns an empty graph for it.

The same passages always give the same graph: NMF starts from a fixed seed, and every sort breaks its ties.

## Errors

`build` returns an `Error` only when there are fewer than 20 passages. Its `passages` field holds how many there were, it displays as "the book has 19 passages; a concept graph needs at least 20", and it implements `std::error::Error`.

## Constants

The constants are fixed in the source, so no argument changes them.

| Constant | Value | Meaning |
|---|---|---|
| `GRAPH_MIN_BOOK_PASSAGES` | 20 | the fewest passages `build` accepts |
| `GRAPH_PHRASE_MAX_WORDS` | 3 | the most content words in a phrase; a longer run gives no phrase |
| `MIN_WORD_LETTERS` | 3 | a word with fewer letters cuts a phrase, like a stopword |
| `GRAPH_CANDIDATE_MIN_PASSAGES` | 3 | the fewest passages a candidate stands in |
| `GRAPH_CANDIDATE_MAX_SHARE` | 0.3 | the largest share of the passages a candidate stands in |
| `MIN_PASSAGES`, `MAX_SHARE` | 2, 0.5 | the fewest passages and the largest share of them a word is in to count for the topics |
| `TOKEN_MAX_BYTES` | 40 | a token of this many bytes or more does not count for the topics |
| `NMF_MAX_ITERATIONS` | 500 | the most NMF iterations |
| `NMF_TOLERANCE` | 1e-4 | NMF stops when its error fell by less than this share since the check ten iterations before |
| `NMF_SEED` | 1 | the seed of the generator that starts NMF |
| `GRAPH_CONCEPTS` | 60 | the most concepts |
| `GRAPH_PARTNERS` | 3 | the most partners a concept keeps |
| `GRAPH_NPMI_FLOOR` | 0.1 | a partner's NPMI must be above this |
| `GRAPH_MIN_SHARED` | 2 | the fewest passages a partner shares |

## Language

The analysis is for English only. Phrases are cut at the 571 SMART stopwords. The topics leave out Lucene's 33 English stopwords and nine question words. Both stem with the English Snowball stemmer of `frostem`, pinned to version 1.20260821.8 so that stems never change. Text in another language is still read, as words are runs of Unicode letters. But its stopwords do not cut phrases and its inflections are not merged, so its concepts are poor.

## Cost

`build` runs on one thread. Its time goes mostly to NMF: an NMF iteration costs about z·k + (n + v)·k² operations for z non-zero cells, v words and k ≈ √n topics, and NMF stops after 500 iterations at most. Links compare every pair of the at most 60 concepts. Memory holds the sparse TF-IDF matrix and the two dense factors, n × k and v × k.

Measured in a release build on an Intel Core Ultra 7 255H:

| Passages | Words | Topics | Time |
|---|---|---|---|
| 166 | 29,000 | 13 | 0.07 s |
| 650 | 100,000 | 25 | 0.42 s |
| 2,402 | 427,000 | 49 | 4.3 s |

The 2,402-passage text peaked at 41 MB for the whole process, its input included.

## License

MIT. See [LICENSE](LICENSE).
