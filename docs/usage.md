# Using concept-map

concept-map maps a long text from its passages. You give `build` the passages in reading order, and it returns a `GraphResult` with three parts: the concepts, phrases from the text that mark its topics; the links, pairs of concepts that share passages more often than chance predicts; and the communities, groups of concepts more linked to each other than to the rest. Every concept and every link lists the ids of its passages, and a passage's id is its index in your input, so each one can be read back in the text. [concept-map.md](concept-map.md) explains how each step works and why.

## Quick start

Add the crate to `Cargo.toml`. It needs Rust 1.92 or later.

```toml
[dependencies]
concept-map = "0.1"
```

`build` refuses fewer than 20 passages, so the example makes 24 short ones: four scenes from *Alice's Adventures in Wonderland*, five times over, and then four passages that mix them. A real text gives its paragraphs instead, as [Splitting a text file into passages](#splitting-a-text-file-into-passages) shows.

```rust
fn main() -> Result<(), concept_map::Error> {
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    for concept in &map.concepts {
        println!(
            "{:>2} {:<17} community {}, passages {:?}",
            concept.id, concept.label, concept.community, concept.passages
        );
    }
    Ok(())
}
```

It prints

```text
 0 crying            community 0, passages [0, 4, 8, 12, 16, 23]
 1 gryphon danced    community 0, passages [0, 4, 8, 12, 16]
 2 lobster quadrille community 0, passages [0, 4, 8, 12, 16]
 3 mock turtle       community 0, passages [0, 4, 8, 12, 16, 22, 23]
 4 dormouse          community 1, passages [1, 5, 9, 13, 17]
 5 hatter            community 1, passages [1, 5, 9, 13, 17, 20, 21]
 6 march hare        community 1, passages [1, 5, 9, 13, 17]
 7 tea               community 1, passages [1, 5, 9, 13, 17]
 8 hearts            community 2, passages [2, 6, 10, 14, 18]
 9 knave             community 2, passages [2, 6, 10, 14, 18]
10 queen             community 2, passages [2, 6, 10, 14, 18]
11 trial             community 2, passages [2, 6, 10, 14, 18, 20, 21]
12 alice             community 3, passages [3, 7, 11, 15, 19, 22]
13 long hall         community 3, passages [3, 7, 11, 15, 19]
14 rabbit hole       community 3, passages [3, 7, 11, 15, 19]
```

Concepts are numbered in the order the text first mentions them. Each scene's phrases form one community, although passages 20 and 21 hold both "hatter" and "trial". The two are not linked: they share those 2 passages, and chance alone predicts 7 × 7 / 24 ≈ 2 shared passages for two concepts that each stand in 7 of the 24.

## Reference

The examples below repeat the quick start's 24 passages wherever they need a map, so that each runs on its own.

### `build`

```rust,ignore
pub fn build(texts: &[&str]) -> Result<GraphResult, Error>
```

Builds the concept map of one text. It cuts the passages into candidate phrases, keeps as concepts the 60 candidates, or fewer, that best predict a passage's topic, links the concepts that share passages more often than chance predicts, and groups the linked concepts into communities.

**Parameters.** `texts` holds the passages of one text, in reading order.

- A passage's id, in every list of passages the map holds, is its index in `texts`.
- `texts` needs at least 20 passages.
- The settings were chosen for passages of whole paragraphs of at most 220 words.
- Every element counts as a passage, an empty one included, and every word in a passage counts, so leave out what you do not want mapped, such as notes, an index or licence text.
- Pass the text as written: `build` splits, lowercases and stems the words itself.
- `build` reads English: its stopword lists and its stemmer are English.

**Returns.** `Ok` with the map, a [`GraphResult`](#graphresult). The same passages always give the same map. A phrase becomes a candidate only when it stands in at least 3 passages and in at most 30% of them, so a text without such a phrase gets an empty map, with no concepts, links or communities, and no error.

**Errors.** `Err(Error)` when `texts` holds fewer than 20 passages, with [`Error::passages`](#error) set to `texts.len()`.

**Panics.** None.

**Example.**

```rust
fn main() {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts).unwrap();
    assert_eq!((map.passages, map.concepts.len()), (24, 15));

    // 19 passages are too few.
    let error = concept_map::build(&texts[..19]).unwrap_err();
    assert_eq!(error.passages, 19);

    // Every phrase stands in all 30 passages, more than 30% of them, so none
    // becomes a concept.
    let empty = concept_map::build(&["the same words everywhere"; 30]).unwrap();
    assert!(empty.concepts.is_empty() && empty.links.is_empty());
}
```

### `GraphResult`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphResult {
    pub passages: u32,
    pub topics: u32,
    pub concepts: Vec<Concept>,
    pub links: Vec<Link>,
    pub communities: Vec<Community>,
}
```

The map that `build` returns. Its parts refer to each other by ids, which are indexes: `map.concepts[id as usize]` is the concept that a link or a community names, and `map.communities[concept.community as usize]` is a concept's community. It derives `Serialize` and `Deserialize`, so serde can save it and load it back under the field names shown.

**Fields.**

- `passages: u32`: how many passages `build` was given.
- `topics: u32`: how many topics `build` sorted the passages into before it chose the concepts, the square root of `passages`, rounded. The map holds only their number, not the topics.
- `concepts: Vec<Concept>`: at most 60 concepts, in the order the text first mentions them. Concepts first mentioned in the same passage are ordered by label. `concepts[i].id` is `i`.
- `links: Vec<Link>`: the links, ordered by `a` and then by `b`. A pair of concepts has at most one link.
- `communities: Vec<Community>`: the communities, largest first. Of two communities of the same size, the one whose first concept the text mentions earlier comes first. `communities[i].id` is `i`, and every concept is in exactly one community.

**Example.**

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    assert_eq!((map.passages, map.topics), (24, 5));
    for link in map.links.iter().take(3) {
        let a = &map.concepts[link.a as usize].label;
        let b = &map.concepts[link.b as usize].label;
        println!("{a} -- {b}");
    }
    Ok(())
}
```

It prints

```text
crying -- gryphon danced
crying -- lobster quadrille
crying -- mock turtle
```

### `GraphResult::to_dot`

```rust,ignore
pub fn to_dot(&self) -> String
```

Writes the map in the DOT language of [Graphviz](https://graphviz.org), whose `dot` command draws it, as [Drawing the map with Graphviz](#drawing-the-map-with-graphviz) shows. The picture follows the map:

- Each concept is a circle in its community's colour, labelled beside it. The first eight communities take eight colours in order, and any later community is grey.
- A circle's radius runs from 3.75 to 12.5 points by the square root of its concept's `passage_count` as a share of the largest, so a concept in more passages gets a larger circle.
- Each link is a line of width 1.25 × (0.8 + 2.4 × `npmi`) points, so a stronger link is a thicker line.
- A legend names each community by its number from 1 and its four concepts with most passages, such as "2 · hatter, dormouse, march hare, tea".
- The layout is Graphviz's force-directed `fdp`, started from the fixed seed `start=7`.
- The background is white.

To find a concept or a community in the DOT, for example to restyle it, look for these names: the concept with id `i` is the node `n{i}`, each community of two or more concepts is a frameless subgraph `cluster_{id}` that keeps its concepts together in the layout, and the legend is the node `legend`.

**Returns.** The whole graph, named `map`, as a `String` that ends with a newline. The same map always gives the same string.

**Panics.** When a community lists a concept id that is not an index into `concepts`. A map from `build` never does; a map edited by hand can, for example after concepts are removed and the rest are not renumbered.

**Example.**

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let dot = concept_map::build(&texts)?.to_dot();
    for line in dot.lines().take(8) {
        println!("{line}");
    }
    Ok(())
}
```

It prints the start of the graph:

```text
graph map {
  graph [layout=fdp, overlap=false, splines=true, sep="+12", K=1.1, maxiter=2000, start=7, forcelabels=true, bgcolor="#ffffff", pad=0.4];
  node [shape=circle, style=filled, fixedsize=true, label="", color="#0f1b33", penwidth=1.2, fontname="Georgia", fontsize=13, fontcolor="#0a1226"];
  edge [color="#0a122659"];
  subgraph cluster_0 { peripheries=0;
    n0 [width=0.329, fillcolor="#f0bf3c", xlabel="crying"];
    n1 [width=0.310, fillcolor="#f0bf3c", xlabel="gryphon danced"];
    n2 [width=0.310, fillcolor="#f0bf3c", xlabel="lobster quadrille"];
```

### `Concept`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    pub id: u32,
    pub label: String,
    pub passage_count: u32,
    pub passages: Vec<u32>,
    pub community: u32,
}
```

A phrase of one to three words that marks one of the text's topics.

**Fields.**

- `id: u32`: its index in `GraphResult::concepts`. Links and communities name a concept by this id.
- `label: String`: the phrase's most frequent form, lowercased, ties going to the form seen first. A concept gathers the inflected forms of its phrase, such as "head" and "heads", because it keys them by their stems.
- `passage_count: u32`: how many passages it stands in, the length of `passages`. It is at least 3 and at most 30% of all passages.
- `passages: Vec<u32>`: the ids of those passages, ascending. A concept stands in a passage only as a phrase of its own, cut off from other words by stopwords or punctuation: "The Hatter left" holds the phrase "hatter left", not "hatter". A search for a concept's words can therefore find more passages than the concept lists.
- `community: u32`: the index of its community in `GraphResult::communities`.

**Example.**

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    println!("{:?}", map.concepts[5]);
    Ok(())
}
```

It prints

```text
Concept { id: 5, label: "hatter", passage_count: 7, passages: [1, 5, 9, 13, 17, 20, 21], community: 1 }
```

### `Link`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    pub a: u32,
    pub b: u32,
    pub npmi: f32,
    pub passage_count: u32,
    pub passages: Vec<u32>,
}
```

Two concepts that share passages more often than chance predicts. Each concept keeps up to three partners: of the concepts that share at least 2 passages with it and have an NPMI above 0.1 with it, those with the highest NPMI, ties going to the lower id. A pair is linked when either concept keeps the other, so a concept can have more than three links, or none.

**Fields.**

- `a: u32`: the id of one concept, the lower of the two.
- `b: u32`: the id of the other concept, so `a < b`.
- `npmi: f32`: the pair's normalized pointwise mutual information, rounded to three decimals. It is 1 when the two concepts only ever stand together and 0 when they share as many passages as chance predicts. A link's NPMI is above 0.1 before rounding and at most 1.
- `passage_count: u32`: how many passages both concepts stand in, the length of `passages`. It is at least 2.
- `passages: Vec<u32>`: the ids of those passages, ascending.

**Example.**

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    println!("{:?}", map.links[2]);
    Ok(())
}
```

It prints the link between "crying", concept 0, and "mock turtle", concept 3:

```text
Link { a: 0, b: 3, npmi: 0.889, passage_count: 6, passages: [0, 4, 8, 12, 16, 23] }
```

"crying" stands only in passages that hold "mock turtle", but "mock turtle" also stands in passage 22, so their NPMI is below 1.

### `Community`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Community {
    pub id: u32,
    pub concepts: Vec<u32>,
}
```

Concepts more linked to each other than to the rest, as the Louvain method finds them on the links' NPMI.

**Fields.**

- `id: u32`: its index in `GraphResult::communities`.
- `concepts: Vec<u32>`: its concepts' ids, ascending. The list is never empty, and a concept without links is a community of its own.

**Example.**

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    for community in &map.communities {
        println!("{community:?}");
    }
    Ok(())
}
```

It prints

```text
Community { id: 0, concepts: [0, 1, 2, 3] }
Community { id: 1, concepts: [4, 5, 6, 7] }
Community { id: 2, concepts: [8, 9, 10, 11] }
Community { id: 3, concepts: [12, 13, 14] }
```

### `Error`

```rust,ignore
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub passages: usize,
}
```

The error `build` returns when it is given fewer than 20 passages. It displays as "the book has 19 passages; a concept map needs at least 20", with the number of passages it was given. It implements `std::error::Error`, so `?` converts it into a `Box<dyn std::error::Error>`.

**Fields.**

- `passages: usize`: how many passages `build` was given.

**Example.**

```rust
fn main() {
    let texts = ["Alice was beginning to get very tired."; 19];
    match concept_map::build(&texts) {
        Ok(map) => println!("{} concepts", map.concepts.len()),
        Err(error) => {
            assert_eq!(error.passages, 19);
            println!("{error}");
        }
    }
}
```

It prints

```text
the book has 19 passages; a concept map needs at least 20
```

## Recipes

### Splitting a text file into passages

Paragraphs make good passages, since the settings were chosen for whole paragraphs of at most 220 words. In a plain-text file a blank line ends a paragraph, so `paragraphs` below collects the runs of lines between blank lines. It joins a paragraph's lines with spaces, so that each passage prints on one line. The joins change nothing for `build`, which reads a line break as a space.

```rust
use std::fs;

/// The paragraphs of `text`, each with its lines joined by spaces.
fn paragraphs(text: &str) -> Vec<String> {
    let mut paragraphs = Vec::new();
    let mut lines: Vec<&str> = Vec::new();
    for line in text.lines().map(str::trim) {
        if !line.is_empty() {
            lines.push(line);
        } else if !lines.is_empty() {
            paragraphs.push(lines.join(" "));
            lines.clear();
        }
    }
    if !lines.is_empty() {
        paragraphs.push(lines.join(" "));
    }
    paragraphs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A sample file, so that the example runs. Read your own file instead.
    let path = std::env::temp_dir().join("scenes.txt");
    let scenes = [
        "The Mock Turtle was crying,\nand the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare\nwere having tea with the Dormouse.",
        "The Queen of Hearts\nwas at the trial of the Knave.",
        "Alice was down the rabbit hole,\nin a long hall.",
    ];
    fs::write(&path, scenes.repeat(6).join("\n\n"))?;

    let text = fs::read_to_string(&path)?;
    let passages = paragraphs(&text);
    // `build` takes string slices, so borrow one from each String.
    let texts: Vec<&str> = passages.iter().map(String::as_str).collect();
    let map = concept_map::build(&texts)?;
    println!("{} passages, {} concepts", map.passages, map.concepts.len());
    println!("passage 1: {}", texts[1]);
    Ok(())
}
```

It prints

```text
24 passages, 15 concepts
passage 1: The Hatter and the March Hare were having tea with the Dormouse.
```

`build` maps everything it receives, so cut what you do not want mapped before you split the file. A Project Gutenberg file, for example, starts with a header and ends with a licence, whose phrases make up most of three of the eight communities in the map of *Alice*.

### Listing each community's concepts

A community holds its concepts' ids, so the labels come from `map.concepts`.

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    for community in &map.communities {
        let labels: Vec<&str> = community
            .concepts
            .iter()
            .map(|&id| map.concepts[id as usize].label.as_str())
            .collect();
        println!("{}: {}", community.id + 1, labels.join(", "));
    }
    Ok(())
}
```

It prints the communities largest first, numbered from 1 as in the legend of `to_dot`:

```text
1: crying, gryphon danced, lobster quadrille, mock turtle
2: dormouse, hatter, march hare, tea
3: hearts, knave, queen, trial
4: alice, long hall, rabbit hole
```

### Reading a concept's passages back

A concept's `passages` are indexes into the slice you gave `build`. Labels are lowercased, so look a concept up by its lowercase label.

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    let hatter = map
        .concepts
        .iter()
        .find(|concept| concept.label == "hatter")
        .ok_or("no concept is labelled \"hatter\"")?;
    for &id in &hatter.passages {
        println!("{id:>2}: {}", texts[id as usize]);
    }
    Ok(())
}
```

It prints

```text
 1: The Hatter and the March Hare were having tea with the Dormouse.
 5: The Hatter and the March Hare were having tea with the Dormouse.
 9: The Hatter and the March Hare were having tea with the Dormouse.
13: The Hatter and the March Hare were having tea with the Dormouse.
17: The Hatter and the March Hare were having tea with the Dormouse.
20: The Hatter was a witness at the trial.
21: At the trial, the Hatter was the first witness.
```

A link's `passages` read back the same way, giving the passages where its two concepts stand together.

### The strongest links

NPMI has the same scale for every pair, 0 for chance and 1 for concepts that only ever stand together, so sorting the links by it puts the strongest first. `sort_by` is stable, so links of equal NPMI keep the map's order, by `a` and then by `b`. The links of one concept are those that name its id as `a` or `b`.

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    let label = |id: u32| map.concepts[id as usize].label.as_str();
    let mut links: Vec<&concept_map::Link> = map.links.iter().collect();
    links.sort_by(|x, y| y.npmi.total_cmp(&x.npmi));

    println!("The strongest links:");
    for link in links.iter().take(3) {
        println!("  {:.3} {} -- {}", link.npmi, label(link.a), label(link.b));
    }

    let turtle = map
        .concepts
        .iter()
        .find(|concept| concept.label == "mock turtle")
        .ok_or("no concept is labelled \"mock turtle\"")?
        .id;
    println!("The strongest links of mock turtle:");
    for link in links.iter().filter(|link| link.a == turtle || link.b == turtle) {
        let partner = if link.a == turtle { link.b } else { link.a };
        println!("  {:.3} {}", link.npmi, label(partner));
    }
    Ok(())
}
```

It prints

```text
The strongest links:
  1.000 gryphon danced -- lobster quadrille
  1.000 dormouse -- march hare
  1.000 dormouse -- tea
The strongest links of mock turtle:
  0.889 crying
  0.785 gryphon danced
  0.785 lobster quadrille
```

### Saving the map as JSON

`GraphResult` and the types in it derive serde's `Serialize` and `Deserialize`, so any serde format can store a map. For JSON, add `serde_json` next to concept-map:

```toml
[dependencies]
concept-map = "0.1"
serde_json = "1"
```

```rust
use concept_map::GraphResult;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    let path = std::env::temp_dir().join("map.json");
    std::fs::write(&path, serde_json::to_string_pretty(&map)?)?;

    let saved: GraphResult = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    println!("{} concepts, {} links", saved.concepts.len(), saved.links.len());
    Ok(())
}
```

It prints `15 concepts, 21 links`. The JSON keeps the Rust field names. With each list cut to its first item, and each item on one line, the file holds

```text
{
  "passages": 24,
  "topics": 5,
  "concepts": [{"id": 0, "label": "crying", "passage_count": 6, "passages": [0, 4, 8, 12, 16, 23], "community": 0}, ...],
  "links": [{"a": 0, "b": 1, "npmi": 0.884, "passage_count": 5, "passages": [0, 4, 8, 12, 16]}, ...],
  "communities": [{"id": 0, "concepts": [0, 1, 2, 3]}, ...]
}
```

### Drawing the map with Graphviz

`to_dot` writes the map as a Graphviz graph, so a program that prints it can pipe it straight into Graphviz's `dot` command.

```rust
fn main() -> Result<(), concept_map::Error> {
    // The quick start's 24 passages.
    let mut texts = [
        "The Mock Turtle was crying, and the Gryphon danced the Lobster Quadrille.",
        "The Hatter and the March Hare were having tea with the Dormouse.",
        "The Queen of Hearts was at the trial of the Knave.",
        "Alice was down the rabbit hole, in a long hall.",
    ]
    .repeat(5);
    texts.extend([
        "The Hatter was a witness at the trial.",
        "At the trial, the Hatter was the first witness.",
        "The Gryphon took Alice to the Mock Turtle.",
        "The Mock Turtle was crying.",
    ]);

    let map = concept_map::build(&texts)?;
    print!("{}", map.to_dot());
    Ok(())
}
```

With [Graphviz](https://graphviz.org) installed, the first command draws the map as an SVG image and the second as a PNG. `-Gdpi` sets the PNG's resolution in dots per inch.

```sh
cargo run | dot -Tsvg -o map.svg
cargo run | dot -Tpng -Gdpi=110 -o map.png
```

The DOT sets `layout=fdp`, so `dot` lays the map out with the force-directed `fdp` rather than in ranks. The DOT is the same for the same map, but Graphviz 2.43, the version these commands were tried with, can place the concepts differently from one run to the next, despite `start=7`.

To keep the DOT, write it to a file with `std::fs::write("map.dot", map.to_dot())` and draw it later with `dot -Tsvg map.dot -o map.svg`.
