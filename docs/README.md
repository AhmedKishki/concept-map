# concept-map documentation

[concept-map.md](concept-map.md) explains how concept-map builds a map from a text's passages: how it reads words, the five steps from candidate phrases to communities with the maths behind each, worked examples on *Alice in Wonderland*, how `to_dot` draws the result, and the limits of the method. [usage.md](usage.md) documents every public function and type of the crate, with recipes for common tasks.

## This documentation, mapped

The picture is concept-map's map of [concept-map.md](concept-map.md) itself, built from its 156 paragraphs and list items. Each colour is a community, a circle grows with its concept's passages, and a line thickens with its NPMI. The communities follow the document's subjects: phrases, the passage-by-word matrix and its topics, NPMI, the graph and its communities, the Louvain method, NMF, the *Alice* examples, and tokens and stems.

![The concept map of concept-map.md](concept-map.png)

[examples/docs_map.rs](../examples/docs_map.rs) takes the document's paragraphs and list items as passages, leaving out its headings, tables, code, formulas and references, and prints the map's `to_dot()`. From the root of the repository, with Graphviz installed, this command builds the map and draws it:

```sh
cargo run --example docs_map | dot -Tpng -Gdpi=110 -o docs/concept-map.png
```

The map is built from the document's own text, so an edit to concept-map.md changes it, and the same command brings the picture up to date.
