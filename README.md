# concept-map

Maps a long text from its passages: the phrases it keeps returning to, which of them go together, and where each one appears.

A concept is a phrase from the text, such as "march hare". Concepts that share passages more often than chance are linked, and linked concepts form communities: in *Alice in Wonderland*, "hatter", "march hare", "dormouse" and "time" form one. Each concept and link lists its passages, so it can be checked against the text. A link says only that two concepts go together, not how, so the map is not a knowledge graph.

## How it works

`build` takes five steps:

1. **Phrases.** It cuts the text at stopwords and punctuation, so "the March Hare said" gives "march hare", and keeps the phrases of up to three words that appear in at least 3 passages and at most 30% of them.
2. **Topics.** It sorts the passages into topics by the words they use (non-negative matrix factorization of TF-IDF weights).
3. **Concepts.** It keeps the 60 phrases that best predict a passage's topic (by mutual information).
4. **Links.** Each concept picks up to three partners with which it shares far more passages than chance predicts, scored by normalized pointwise mutual information (NPMI): 1 for concepts always found together, 0 for chance. Either pick makes a link.
5. **Communities.** It groups the concepts that link more to each other than to the rest (the Louvain method).

[docs/concept-map.md](docs/concept-map.md) explains each step from the basics.

## Its own documentation, mapped

Run on the prose of [docs/concept-map.md](docs/concept-map.md), 118 paragraphs and list items, the crate finds eight communities. They line up with the doc's subjects: the Louvain method, the passage-by-word matrix, the *Alice* examples, graphs, NMF, candidate phrases and information theory, plus one concept without links.

- community, concepts, links, stand, number, ties, bound, order, five-node graph, modularity, phase
- subject, text, step, words, toy text, tokens, stem, cell, column, row
- march hare, score, hatter, alice, topic, tea, time, dormous
- graph, joins, sum, weight, node, degrees, gains, joining node
- holds, error, entry, lee, seung, iteration, ratio, update
- phrase, cut, digit, letters, run, smart stopwords, content words
- pair, entropy, average, logarithm, probability, rarer, share
- measures

`cargo run --example docs_map` rebuilds it.

## Install

The crate is not on crates.io yet, so add it from git. It needs Rust 1.92 or later.

```toml
[dependencies]
concept-map = { git = "https://github.com/AhmedKishki/concept-map" }
```

## Usage

```rust
fn main() -> Result<(), concept_map::Error> {
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

    let map = concept_map::build(&texts)?;
    for community in &map.communities {
        let labels: Vec<&str> = community
            .concepts
            .iter()
            .map(|&id| map.concepts[id as usize].label.as_str())
            .collect();
        println!("{}", labels.join(", "));
    }
    let link = &map.links[0];
    println!(
        "{} and {}: NPMI {}, passages {:?}",
        map.concepts[link.a as usize].label,
        map.concepts[link.b as usize].label,
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

At least 20 passages of one text, in reading order; the settings were chosen for passages of whole paragraphs of at most 220 words. Raw text needs no preparation, but everything in it counts, so remove notes, an index or licence text that should not be mapped. The stopwords and the stemmer are English, so other languages give poor concepts.

## Output

`build` returns a `GraphResult`, which serde can serialize:

- `concepts`, each with its `label`, `passages` and `community`;
- `links`, each joining concepts `a` and `b`, with their `npmi` and shared `passages`;
- `communities`, each with its `concepts`, largest first;
- `passages` and `topics`, the number of each.

A passage's id is its index in your input, and a concept's id its index in `concepts`. `build` fails only on fewer than 20 passages.

## Cost

`build` runs on one thread. On an Intel Core Ultra 7 255H it took 0.07 s for 166 passages, 0.42 s for 650 and 4.3 s for 2,402.

## Similar tools

[VOSviewer](https://www.vosviewer.com/), a desktop application, and [InfraNodus](https://infranodus.com/), a web service, also map a text's terms by co-occurrence and cluster them by modularity, while [GraphRAG](https://arxiv.org/abs/2404.16130) has a language model extract entities and then groups them. This crate is a Rust library with no model of any kind, so the same passages always give the same map, and nothing in it is generated.

## License

MIT. See [LICENSE](LICENSE).
