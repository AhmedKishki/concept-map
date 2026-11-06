# Changelog

Every change to concept-map gets a version and an entry here, newest first. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html): documentation and fixes that keep the behaviour bump the patch number, and new features or changed behaviour bump the minor number while the crate is at 0.x. The entries follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.2.0] - 2026-11-06

### Changed

- The 60 concepts are now moderately linked. `build` drops both phrases of every pair of concepts whose NPMI is 0.5 or more, takes the next best candidates by mutual information in their places, and repeats until no pair is that strongly linked or the candidates have run out. A dropped phrase does not come back, and a text can end with fewer than 60 concepts. Every link in a map is therefore above an NPMI of 0.1 and below 0.5. Texts whose subjects always stand together lose them: in the map of *Alice* the king, the queen, the hatter, the march hare, the dormouse, the gryphon and the mock turtle are dropped, and so are most of the Project Gutenberg licence's phrases. The same text gives a different map than in 0.1.1, so a program that saved maps should build them again.
- The documentation's examples use eight characters in a ring of neighbouring pairs, whose links are moderate, and the walk-through of *Alice* in `docs/concept-map.md` shows the new step, with its numbers and pictures redrawn.

## [0.1.1] - 2026-11-05

### Added

- `docs/usage.md`, a usage reference: `build`, `to_dot` and every result type and field with what it means, what comes back, its errors and panics and an example with its real output, and recipes for splitting a file into passages, reading a concept's passages back, saving JSON and drawing with Graphviz. Every example runs as a doctest.
- A `missing_docs` warning for any undocumented public item, and documentation of the panic `to_dot` raises when a community names a concept id that is not in `concepts`.

### Fixed

- The documentation said that `to_dot`'s fixed seed makes the same map give the same picture. Graphviz 2.43 can lay out the same DOT differently from one run to the next, so it now says that the DOT is the same and the picture may differ.

## [0.1.0] - 2026-11-05

### Added

- `build`, which turns a text's passages into concepts, NPMI links and Louvain communities, and `GraphResult::to_dot`, which writes the map for Graphviz.
- `docs/concept-map.md`, which explains each step from the basics with worked examples, and `examples/docs_map.rs`, which maps that document.

[0.2.0]: https://github.com/AhmedKishki/concept-map/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/AhmedKishki/concept-map/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/AhmedKishki/concept-map/releases/tag/v0.1.0
