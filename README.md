# concept-graph

Builds a map of a long text from its passages: the phrases it keeps returning to, which of them go together, and where each one appears.

A concept is a phrase from the text, such as "march hare" or "transcendental realism". Two concepts are linked when they share passages more often than chance predicts, and linked concepts form communities: in *Alice in Wonderland*, "hatter", "march hare", "dormouse" and "time" form one, while "king", "queen", "jury" and "trial" belong to another. Every concept and every link lists its passages, so you can read the text behind it.

## How it works

`build` takes five steps:

1. **Phrases.** It cuts each passage at stopwords such as "the" and "said" and at punctuation, so "the March Hare said" gives the phrase "march hare". Phrases of up to three words that appear in at least 3 passages, but in no more than 30% of them, go on to step 3.
2. **Topics.** It sorts the passages into topics by the words they use (TF-IDF weights, factorized by non-negative matrix factorization).
3. **Concepts.** It keeps the 60 phrases that best tell those topics apart (by mutual information).
4. **Links.** Each concept chooses up to three partners with which it shares far more passages than chance predicts, and either choice makes a link, so a popular concept can have more than three. The score, normalized pointwise mutual information (NPMI), is 1 for two concepts that always appear together and 0 for two that share passages only by chance.
5. **Communities.** It splits the linked concepts into groups that are more linked to each other than to the rest (the Louvain method).

[docs/concept-graph.md](docs/concept-graph.md) explains each step from the basics, with worked examples on *Alice in Wonderland*.

## Install

The crate is not on crates.io yet, so add it from git:

```toml
[dependencies]
concept-graph = { git = "https://github.com/AhmedKishki/concept-graph" }
```

It needs Rust 1.92 or later.

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

- The passages of one text, in reading order, at least 20 of them. The settings were chosen for passages of whole paragraphs of at most 220 words.
- Plain text, which needs no preparation, because `build` splits, lowercases and stems the words itself. Everything in it counts, though, so remove notes, an index, running heads or licence text that you do not want in the graph.
- English. The stopword lists and the stemmer are English, so other languages give poor concepts.

## Output

`build` returns a `GraphResult`, which serde can serialize:

- `concepts`, each with its `label`, its `passages` and its `community`;
- `links`, each joining two concepts `a` and `b`, with their `npmi` and the `passages` they share;
- `communities`, each with its `concepts`, the largest community first;
- `passages` and `topics`, the number of passages and of topics.

Ids are indexes: a passage's id is its index in your input, and a concept's id is its index in `concepts`. `build` fails only when there are fewer than 20 passages.

## Cost

`build` runs on one thread. On an Intel Core Ultra 7 255H it took 0.07 s for 166 passages, 0.42 s for 650 and 4.3 s for 2,402, and the largest peaked at 41 MB.

## Similar tools

Maps of terms built from co-occurrence are an established idea. [VOSviewer](https://www.vosviewer.com/) maps the noun phrases of a set of documents and clusters them by modularity, [InfraNodus](https://infranodus.com/) builds a network of a text's words and finds its topics with the Louvain method, and [GraphRAG](https://arxiv.org/abs/2404.16130) has a language model extract entities before it groups them into communities. This crate makes different trade-offs:

- It needs no model, language model or training data, and it runs as a Rust library inside your program.
- It is deterministic: the same passages always give the same graph.
- Its concepts are phrases from the text and its links rest on counted passages, so nothing in the graph is generated.
- It keeps the phrases that best tell the text's topics apart, so a phrase that is merely frequent does not become a concept.

## License

MIT. See [LICENSE](LICENSE).
