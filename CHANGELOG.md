# Changelog

Every change to concept-map gets a version and an entry here, newest first. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html): documentation and fixes that keep the behaviour bump the patch number, and new features or changed behaviour bump the minor number while the crate is at 0.x. The entries follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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

[0.1.1]: https://github.com/AhmedKishki/concept-map/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/AhmedKishki/concept-map/releases/tag/v0.1.0
