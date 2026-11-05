# The concept graph

The concept graph shows which subjects a text keeps returning to and which of them it treats together. A concept is a phrase from the text, such as "march hare" or "transcendental realism". Two concepts are linked when they stand in the same passages more often than chance predicts, and linked concepts fall into communities. Every concept and every link carries the ids of its passages, so each one can be checked against the text.

The graph gives a map of a text before you know what to search for: its recurring subjects, which of them go together, and where in the text each one appears. It is built from the passages alone, without an outline, a model or training data, and takes seconds.

## The pipeline

`build` in `src/lib.rs` turns the passages into a graph in five steps.

```mermaid
flowchart TD
    P["Passages<br/>in reading order"] --> C["1. Candidate phrases<br/>RAKE cuts, SMART stopwords<br/>in 3 passages or more, 30% at most"]
    P --> T["2. Topics<br/>TF-IDF, NMF of rank round(√n)"]
    C --> S["3. Concepts<br/>the 60 candidates with the most<br/>mutual information with the topics"]
    T --> S
    S --> L["4. Links<br/>NPMI above 0.1, 2 shared passages or more,<br/>3 best partners per concept"]
    L --> M["5. Communities<br/>Louvain on the NPMI weights"]
    M --> G["GraphResult"]
```

`build` takes the caller's passages as a slice of texts in reading order, and a passage's id is its index in that slice. A passage is the unit of co-occurrence. Two concepts occur together when both stand in one passage, wherever they are inside it. The constants were chosen for passages of whole paragraphs of at most 220 words.

Below, n is the number of passages. `build` refuses fewer than `GRAPH_MIN_BOOK_PASSAGES` = 20 passages, with an error such as "the book has 19 passages; a concept graph needs at least 20".

The worked examples below use the Project Gutenberg EPUB of Lewis Carroll's *Alice's Adventures in Wonderland*, cut into 166 passages of whole paragraphs of at most 220 words, with the chapter headings left out.

### Words and stems

Steps 1 and 2 read words with the crate's own analyzer in `src/analysis.rs`, which has two pipelines. Both split the text into tokens at every character that is neither a letter nor a digit, as `char::is_alphanumeric` decides, so a token is a maximal run of letters and digits. Both lowercase each token and stem it with the English Snowball stemmer of the `frostem` crate, which `Cargo.toml` pins to version 1.20260821.8 so that a stem never changes under a graph. An ASCII token is lowercased as ASCII and any other token character by character, so a capital sigma becomes σ even at the end of a word, where `str::to_lowercase` writes ς.

- `KEY` drops nothing. Step 1 keys its phrases with it through `stem_word`.
- `STEMMED` drops more. Before lowercasing it drops every token of `TOKEN_MAX_BYTES` = 40 bytes or more, such as a URL or an encoded blob. After lowercasing it drops Lucene's 33 English stopwords (a, an, and, are, as, at, be, but, by, for, if, in, into, is, it, no, not, of, on, or, such, that, the, their, then, there, these, they, this, to, was, will, with) and nine question words (how, what, when, where, which, who, whom, whose, why). Step 2 counts the tokens that remain.

The two pipelines give, token for token, what tantivy 0.27 gives for `SimpleTokenizer` followed by `RemoveLongFilter::limit(40)`, `LowerCaser`, `StopWordFilter::new(Language::English)`, a `StopWordFilter` of the question words and `Stemmer::new(Language::English)` for `STEMMED`, and by `LowerCaser` and the stemmer alone for `KEY`. tantivy 0.27 stems with `frostem` too.

### Step 1: candidate phrases

`candidates` in `src/phrases.rs` finds candidates as RAKE does: a candidate is a run of words between stopwords and punctuation. The text is lowercased and read word by word.

- A word is a run of letters that may hold `-`, `'` or `’` between two letters, so "labor-power" and "naïve" stay whole.
- A final possessive `'s` or `’s` is dropped, so "Kant’s" becomes "kant".
- A word that still holds an apostrophe, a contraction such as "doesn't", cuts the run.
- Any other character that is neither a letter nor whitespace cuts the run: punctuation, digits and symbols. Whitespace, line breaks included, only separates words.
- A word cuts the run when it is one of the 571 SMART stopwords in `src/smart_stopwords.txt` or has fewer than `MIN_WORD_LETTERS` = 3 letters.

The words that do not cut are content words. A run of 1 to `GRAPH_PHRASE_MAX_WORDS` = 3 content words is a phrase. A longer run is dropped whole, not split. The unit tests pin examples. "The transcendental realism of science, in 1975, rejects positivism." gives "transcendental realism", "science" and "rejects positivism". "Kant’s critique doesn't answer Hume's problem" gives "kant critique" and "answer hume problem". "red green blue yellow" gives nothing.

A phrase is keyed by its words stemmed with the `KEY` pipeline (`stem_word` in `src/analysis.rs`, which stems a hyphenated word part by part and joins the parts with `-`), so "head" and "heads" share a key. Its label is the lowercased form it most often has, ties going to the form seen first.

A phrase becomes a candidate when it stands in at least `GRAPH_CANDIDATE_MIN_PASSAGES` = 3 passages and in at most `GRAPH_CANDIDATE_MAX_SHARE` = 0.3 of them. Both limits count passages, not occurrences. In the unit test with 10 passages, "mock turtle" stands in 3 and is kept, "gryphon" stands in 2 and is too rare, and "queen" stands in 4 and is too common.

A phrase stands in a passage only as a run of its own. "turtle" inside "mock turtle" does not count for a phrase "turtle". A search for a concept's words can therefore find more passages than the concept has.

RAKE goes on to score each candidate by the degree and frequency of its words. This crate takes only RAKE's way of cutting candidates and chooses among them in step 3.

### Step 2: topics

`passage_topics` in `src/topics.rs` sorts the passages into topics by non-negative matrix factorization (NMF). The output does not list the topics, only their number; step 3 uses them to choose concepts.

The number of topics k is `topic_count(n)`: the square root of n, rounded, and at least 2. The 166 passages of *Alice* give 13 topics.

The matrix X has a row per passage and a column per word. Its words are the `STEMMED` tokens of the analyzer: lowercased and stemmed, without the English stopwords, the question words, or tokens of 40 bytes or more. The SMART list of step 1 plays no part here. A word gets a column when it is in at least `MIN_PASSAGES` = 2 passages and in at most `MAX_SHARE` = 0.5 of them; a word in more passages is, as the code's comment puts it, in every topic. Columns are in alphabetical order, so the same passages always give the same matrix. A cell holds

$$x = (1 + \ln \mathrm{tf}) \cdot \left( \ln \frac{1 + n}{1 + \mathrm{df}} + 1 \right)$$

where tf is how often the word occurs in the passage and df is how many passages hold it. Each row is then scaled to unit length.

NMF approximates X by WH, where W is n × k, H is k × (number of words), and both are non-negative. The rank of the factorization is the number of topics. `nmf` lowers the squared error ‖X − WH‖² by Lee and Seung's multiplicative updates, H first and then W in each iteration:

$$H \leftarrow H \circ \frac{W^\top X}{W^\top W H + \epsilon} \qquad W \leftarrow W \circ \frac{X H^\top}{W H H^\top + \epsilon}$$

The product and the division are taken entry by entry, and `EPSILON` = 1e-10 prevents division by zero. Each factor is non-negative, so W and H stay non-negative without any clipping, and Lee and Seung (2001) prove that these updates never increase the error.

The starting values come from a SplitMix64 generator seeded with `NMF_SEED` = 1. Each entry of W and H is uniform on [0, 2√(x̄/k)), where x̄ is the mean cell of X with zeros included, so that WH starts with the same mean as X. A fixed seed gives the same start on every machine.

The loop runs at most `NMF_MAX_ITERATIONS` = 500 times. Every `CHECK_EVERY` = 10 iterations it computes the error, without forming WH, and it stops when the error fell by less than `NMF_TOLERANCE` = 1e-4 of its value at the previous check. The earliest stop is therefore after 20 iterations.

A passage's topic is the column of its row of W with the largest value, ties going to the lower topic. When no word passes the column limits, every passage is in topic 0.

### Step 3: choosing concepts

`select` in `src/lib.rs` keeps the candidates that best predict a passage's topic. For a candidate, let G say whether a passage holds it and T be the passage's topic. The candidate's score is their mutual information in nats:

$$I(T;G) = \sum_{t} \sum_{g \in \{0,1\}} p(t,g) \ln \frac{p(t,g)}{p(t)\,p(g)}$$

Every probability is a share of the n passages: p(t) is the share in topic t, p(g = 1) the share that hold the phrase, p(t, 1) the share that are in topic t and hold it, and p(t, 0) the share in topic t that do not. A term without passages counts as 0. The unit test pins hand values: with four passages in topics 0, 0, 1, 1, a phrase in the first two scores ln 2 and a phrase in the first and the third scores 0.

The score is high for a phrase whose passages fall in few topics. It can never exceed the entropy of G, so a phrase in only 3 passages scores low however concentrated it is.

The `GRAPH_CONCEPTS` = 60 highest scores become the concepts, ties going to the phrase in more passages, then to the lower key. The concepts are then sorted by their first passage, ties by label, and numbered from 0 in that order.

### Step 4: links

`links` in `src/lib.rs` scores each pair of concepts by normalized pointwise mutual information (NPMI). With $d_a$ and $d_b$ the numbers of passages each concept stands in and $s$ the number both stand in, `npmi` computes

$$\mathrm{npmi}(a,b) = \frac{\ln \dfrac{s\,n}{d_a\,d_b}}{-\ln \dfrac{s}{n}}$$

This is $\ln \frac{p(a,b)}{p(a)\,p(b)}$ divided by $-\ln p(a,b)$, with each probability a share of passages. It is 1 when the two only ever stand together ($s = d_a = d_b$), 0 when they share as many passages as independence predicts ($s = d_a d_b / n$), and negative when they share fewer. The unit test pins both values: `npmi(2, 2, 2, 10)` is 1 and `npmi(4, 5, 2, 10)` is 0.

In *Alice*, "hatter" stands in 18 passages, "march hare" in 17, and they share 16 of the 166:

$$\frac{\ln (16 \cdot 166 / (18 \cdot 17))}{\ln (166 / 16)} = \frac{\ln 8.68}{\ln 10.38} = \frac{2.161}{2.339} = 0.924$$

Each concept keeps up to `GRAPH_PARTNERS` = 3 partners. The partners it can keep are the concepts that share at least `GRAPH_MIN_SHARED` = 2 passages with it and have an NPMI above `GRAPH_NPMI_FLOOR` = 0.1; it keeps those with the highest NPMI, ties going to the lower id. A link exists when either concept keeps the other. A concept that many others keep can therefore have more than three links ("king" in *Alice* has eight), and a concept that keeps no partner and is kept by none has no link.

The share limit of step 1 keeps the formula defined. Both concepts stand in at most 0.3n passages, so s is at most 0.3n and the denominator is at least ln(1/0.3) ≈ 1.20, never 0. The minimum of 2 shared passages keeps the logarithms finite.

The output rounds NPMI to three decimals. Step 5 uses the unrounded value.

### Step 5: communities

`louvain` in `src/communities.rs` finds communities by the Louvain method of Blondel, Guillaume, Lambiotte and Lefebvre. Its graph has the concepts as nodes and the links as undirected edges weighted by their NPMI. The method looks for the division into communities with the highest modularity:

$$Q = \frac{1}{2m} \sum_{i,j} \left( A_{ij} - \frac{k_i k_j}{2m} \right) \delta(c_i, c_j)$$

Here $A_{ij}$ is the weight between i and j, $k_i$ is the sum of i's weights, m is the sum of all edge weights, and $\delta(c_i, c_j)$ is 1 when i and j are in one community and 0 otherwise. Q compares the weight inside the communities with the weight a random graph with the same weighted degrees would put there.

The method alternates two phases. In the first, `move_nodes`, every node starts in a community of its own. The nodes are visited in index order, which at the first level is the order in which the text first mentions the concepts. A node i is taken out of its community. For that community and for each community C that i has a link into, the gain in modularity from putting i into C is

$$\Delta Q = \frac{k_{i,C}}{m} - \frac{\Sigma_C \, k_i}{2m^2}$$

where $k_{i,C}$ is the weight of i's links into C and $\Sigma_C$ is the sum of the degrees of the nodes in C, i excluded. The node joins the community with the largest gain. Its own community is the first best, and another replaces the best so far only when its gain is larger by more than `MIN_GAIN` = 1e-7, so that rounding cannot make a node move back and forth. Sweeps over all nodes repeat until a sweep moves no node.

In the second phase, `aggregate`, each community becomes one node. The links between two communities add up to one link, and the links inside a community become a self-loop holding twice their weight, as the adjacency matrix would. The first phase then runs on this smaller graph, and the two phases repeat until a first phase moves no node.

There is no resolution parameter, so the gains are those of the plain modularity above, and there is no limit on the number of passes. A concept without links stays a community of its own, and a graph without links has one community per concept.

`build` orders the communities largest first, ties going to the community whose first concept the text mentions earlier, and numbers them in that order. In *Alice*, "mock turtle" and "gryphon" are in one community, and so are "hatter" and "march hare".

### What the graph reads

The graph filters nothing. It reads every passage it is given, so notes, a contents page, a bibliography, an index, running heads and licence text count like the body of the text. A caller who wants them left out removes them from the passages before calling `build`.

## Output

`GraphResult` in `src/lib.rs` holds the graph:

| Field | What it holds |
|---|---|
| `passages` | n, the number of passages |
| `topics` | k, the number of topics of step 2 |
| `concepts[]` | `id`, `label`, `passage_count`, `passages` (ids in reading order) and `community` |
| `links[]` | `a` (the lower id), `b`, `npmi` (three decimals), and the `passage_count` and `passages` of the passages both concepts stand in |
| `communities[]` | `id` and `concepts` (ids, ascending) |

Concepts come in the order the text first mentions them, links sorted by `a` and then `b`, and communities largest first. Every passage id is an index into the slice `build` was given, and a link's passages are passages of both its concepts.

A text in which no phrase is a candidate gets an empty graph, with no concepts, links or communities.

## Why it works this way

Phrases. A concept is named in the text's own words and backed by the passages that hold them, so every concept can be checked by reading. Cutting at stopwords and punctuation keeps a multi-word name such as "march hare" or "mock turtle" whole without a parser or a model. Stemmed keys gather the inflected forms of a phrase under one concept.

Topics. Frequency alone favours phrases spread evenly through a text. Mutual information with the topics favours phrases that tell its parts apart: of two phrases in the same number of passages, the one whose passages fall in fewer topics scores higher. The share limits remove what is everywhere before either count is made.

NPMI rather than shared counts. The number of passages two concepts share grows with how common each of them is. In *Alice*, "time" stands in 47 passages and shares 8 with "hatter", half as many as "march hare" shares with it. Yet chance alone predicts 47 · 18 / 166 ≈ 5.1 shared passages for "time", so its NPMI with "hatter" is 0.149 against 0.924 for "march hare". Plain PMI errs the other way and favours rare pairs: two concepts in 3 passages each that share 2 have a PMI of ln 36.9 = 3.61, above the 2.16 of "hatter" and "march hare", but an NPMI of 0.82, below their 0.92. Dividing by −ln p(a,b) bounds the score between −1 and 1 with fixed meanings at −1, 0 and 1 (Bouma 2009), so one floor of 0.1 serves every pair in every text.

Communities. A community is a set of concepts "more linked to each other than to the rest", as the comment on `Community` says. Modularity measures that against what the concepts' degrees alone predict, so a concept with many links does not draw every concept it touches into its community. In *Alice*, "hatter", "march hare" and "dormouse" form one community, and "king", "jury", "trial", "court" and "slates" another.

Determinism. The same passages give the same graph on every machine. The NMF start comes from a fixed seed, the matrix columns are in word order, Louvain visits nodes in index order, and every sort breaks ties by count, id or key.

## Limits

- The input needs at least 20 passages.
- A graph has at most 60 concepts, however long the text. In a long text more subjects compete for the 60 places, and a subject whose phrases score lower is left out however often it occurs.
- The method is for English: the SMART stopwords, Lucene's stopwords, the question words and the stemmer are English.
- A phrase is any run of up to three content words, not a noun phrase. In *Alice* "thought alice" is a concept, because "said" is a SMART stopword and "thought" is not.
- A name that holds a stopword, a digit or a word of fewer than three letters is split or lost: "queen of hearts" gives "queen" and "hearts". A run of more than three content words gives nothing.
- The 30% limit can remove a text's most frequent subject. In *Alice* "alice" stands alone in more than 100 of the 166 passages, where the limit allows 49, so the heroine is not a concept.
- Only stand-alone mentions count. Pronouns, synonyms and a concept's words inside a longer phrase add nothing to its passages, and "hatter" and "mad hatter" would be two concepts.
- Whitespace never cuts a phrase, line breaks included. A heading, paragraph or list item that ends without punctuation therefore runs into the first words of whatever follows it in the passage.
- A link says only that two concepts share passages more often than chance predicts. It has no direction or kind, and a passage counts as one place whatever the distance between the two inside it.
- Each passage belongs to exactly one topic, and the number of topics is fixed at √n rounded. The concepts are chosen by these topics, so a different factorization would choose different concepts.
- Louvain returns a local optimum of modularity, the one its fixed visiting order reaches.
- Apparatus counts like text. The Project Gutenberg header and licence of *Alice* (passage 0 and passages 149 to 165) give concepts such as "project gutenberg", "electronic works", "www", "gutenberg" and "org", and three of the eight communities consist mostly of them. "www", "gutenberg" and "org" are linked with an NPMI of 1, because the dots of www.gutenberg.org cut it into three phrases that always stand together.

## References

- Stuart Rose, Dave Engel, Nick Cramer and Wendy Cowley. "Automatic Keyword Extraction from Individual Documents". In Michael W. Berry and Jacob Kogan (eds.), *Text Mining: Applications and Theory*, John Wiley & Sons, 2010, pp. 1–20. RAKE, whose candidate step is step 1.
- David D. Lewis, Yiming Yang, Tony G. Rose and Fan Li. "RCV1: A New Benchmark Collection for Text Categorization Research". *Journal of Machine Learning Research* 5 (2004), pp. 361–397. Its Appendix 11 republishes the SMART stopword list in `src/smart_stopwords.txt`.
- Daniel D. Lee and H. Sebastian Seung. "Learning the parts of objects by non-negative matrix factorization". *Nature* 401 (1999), pp. 788–791. NMF, applied there to images and to text.
- Daniel D. Lee and H. Sebastian Seung. "Algorithms for Non-negative Matrix Factorization". In *Advances in Neural Information Processing Systems 13* (NIPS 2000), MIT Press, 2001, pp. 556–562. The multiplicative updates for the squared error used in step 2.
- Guy L. Steele Jr., Doug Lea and Christine H. Flood. "Fast Splittable Pseudorandom Number Generators". In *Proceedings of the 2014 ACM International Conference on Object Oriented Programming Systems Languages & Applications* (OOPSLA 2014), ACM, 2014. SplitMix64, the generator that seeds NMF.
- Gerlof Bouma. "Normalized (Pointwise) Mutual Information in Collocation Extraction". In *From Form to Meaning: Processing Texts Automatically, Proceedings of the Biennial GSCL Conference 2009*, Gunter Narr, 2009, pp. 31–40. NPMI, used in step 4.
- Mark E. J. Newman and Michelle Girvan. "Finding and evaluating community structure in networks". *Physical Review E* 69 (2004), 026113. Modularity.
- Vincent D. Blondel, Jean-Loup Guillaume, Renaud Lambiotte and Etienne Lefebvre. "Fast unfolding of communities in large networks". *Journal of Statistical Mechanics: Theory and Experiment* (2008), P10008. The Louvain method of step 5.
