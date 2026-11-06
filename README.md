# concept-map

Maps a long text from its passages: the phrases it keeps returning to, which of them go together, and where each one appears.

A concept is a phrase from the text, such as "march hare". Concepts that share passages more often than chance are linked, and linked concepts form communities. Every concept and link lists the passages it comes from.

## Alice in Wonderland, mapped

The map of the Project Gutenberg EPUB of *Alice's Adventures in Wonderland*, in 166 passages. Each colour is a community, a circle grows with its concept's passages, and a line thickens with its NPMI. The characters, which always stand together, are too strongly linked to stay, so the map holds ordinary words of the story, such as "head", "looked", "cat" and "voice".

![The concept map of Alice's Adventures in Wonderland](docs/alice.png)

## How it works

`build` takes five steps:

1. **Phrases.** It cuts the text at stopwords and punctuation, so "the March Hare said" gives "march hare", and keeps the phrases of up to three words that appear in at least 3 passages and at most 30% of them.
2. **Topics.** It sorts the passages into topics by the words they use (non-negative matrix factorization of TF-IDF weights).
3. **Concepts.** It keeps the 60 phrases that best predict a passage's topic (by mutual information), leaving out any that is linked to another at an NPMI of 0.5 or more.
4. **Links.** Each concept picks up to three partners with which it shares more passages than chance predicts, scored by normalized pointwise mutual information (NPMI): 1 for concepts always found together, 0 for chance. Either pick makes a link. A pair at an NPMI of 0.5 or more is too strongly linked: step 3 drops both phrases and takes the next best in their places, so every link in the map is a moderate one.
5. **Communities.** It groups the concepts that link more to each other than to the rest (the Louvain method).

[docs/concept-map.md](docs/concept-map.md) explains each step from the basics, and [docs/README.md](docs/README.md) shows the crate's map of that explanation. [docs/usage.md](docs/usage.md) documents every public function and type, with recipes for common tasks.

## Install

Add it to `Cargo.toml`; it needs Rust 1.92 or later.

```toml
[dependencies]
concept-map = "0.2"
```

## Usage

```rust
fn main() -> Result<(), concept_map::Error> {
    // 24 passages in reading order: 20 sentences that each name two
    // neighbouring characters, and four that name one.
    let mut texts = [
        "The Hatter and the March Hare.",
        "The March Hare and the Dormouse.",
        "The Dormouse and the Queen.",
        "The Queen and the Knave.",
    ]
    .repeat(3);
    texts.extend(
        [
            "The Knave and the Gryphon.",
            "The Gryphon and the Mock Turtle.",
            "The Mock Turtle and the Caterpillar.",
            "The Caterpillar and the Hatter.",
        ]
        .repeat(2),
    );
    texts.extend(["The Hatter!", "The Dormouse!", "The Queen!", "The Gryphon!"]);

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
hatter, march hare, dormouse
gryphon, mock turtle, caterpillar
queen, knave
hatter and march hare: NPMI 0.333, passages [0, 4, 8]
```

## Input

`build` takes the passages of one text in reading order, at least 20 of them; fewer return an `Error`. The settings were chosen for passages of whole paragraphs of at most 220 words. Pass the text you want mapped, leaving out notes, an index or licence text: `build` splits, lowercases and stems the words itself, and everything it receives counts. It reads English, with English stopword lists and an English stemmer.

## Output

`build` returns a `GraphResult`, which serde can serialize:

- `concepts`, each with its `label`, `passages` and `community`;
- `links`, each joining concepts `a` and `b`, with their `npmi` and shared `passages`;
- `communities`, each with its `concepts`, largest first;
- `passages` and `topics`, the number of each.

A passage's id is its index in your input, and a concept's id its index in `concepts`. `map.to_dot()` writes the map in Graphviz's DOT language, which `dot -Tpng` or `dot -Tsvg` draws as in the picture above.

## Cost

`build` runs on one thread. On an Intel Core Ultra 7 255H it took 0.07 s for 166 passages, 0.42 s for 650 and 4.3 s for 2,402.

## Similar tools

[VOSviewer](https://www.vosviewer.com/), a desktop application, and [InfraNodus](https://infranodus.com/), a web service, map a text's terms by co-occurrence and cluster them by modularity, and [GraphRAG](https://arxiv.org/abs/2404.16130) has a language model extract entities and group them. concept-map does this from the text alone, as a Rust library, and the same passages always give the same map.

## License

MIT. See [LICENSE](LICENSE).
