//! A book's concept map: the phrases that mark its topics, which of them
//! share passages more often than chance, and the communities they form.

mod analysis;
mod communities;
mod phrases;
mod topics;

use std::collections::BTreeSet;
use std::fmt::{self, Write};

use serde::{Deserialize, Serialize};

use communities::louvain;
use phrases::{Candidate, candidates};
use topics::{passage_topics, topic_count};

const GRAPH_CONCEPTS: usize = 60;
const GRAPH_PARTNERS: usize = 3;
const GRAPH_NPMI_FLOOR: f64 = 0.1;
const GRAPH_MIN_SHARED: usize = 2;
const GRAPH_MIN_BOOK_PASSAGES: usize = 20;

/// A book's concept map.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GraphResult {
    /// How many passages the book has.
    pub passages: u32,
    /// How many topics the concepts were chosen with.
    pub topics: u32,
    /// The concepts, in the order the book first mentions them.
    pub concepts: Vec<Concept>,
    /// The links between concepts, by `a` then `b`.
    pub links: Vec<Link>,
    /// The communities, largest first.
    pub communities: Vec<Community>,
}

/// A phrase that marks one of the book's topics.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Concept {
    /// Its index in `concepts`.
    pub id: u32,
    /// The form the book most often writes it in, lowercased.
    pub label: String,
    /// How many passages it stands in as a phrase of its own, cut off from
    /// other words by stopwords or punctuation.
    pub passage_count: u32,
    /// Those passages' ids, in reading order. A passage's id is its index in
    /// the texts `build` was given.
    pub passages: Vec<u32>,
    /// The index of its community in `communities`.
    pub community: u32,
}

/// Two concepts that share passages more often than chance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The concept with the lower id.
    pub a: u32,
    /// The concept with the higher id.
    pub b: u32,
    /// Normalized pointwise mutual information: 1 when the two only ever
    /// occur together, 0 when they share passages as often as chance.
    pub npmi: f32,
    /// How many passages both concepts stand in.
    pub passage_count: u32,
    /// Those passages' ids, in reading order. A passage's id is its index in
    /// the texts `build` was given.
    pub passages: Vec<u32>,
}

/// Concepts more linked to each other than to the rest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Community {
    /// Its index in `communities`.
    pub id: u32,
    /// Its concepts' ids, ascending.
    pub concepts: Vec<u32>,
}

/// One colour per community, in order; the communities past these are grey.
const COMMUNITY_COLOURS: [&str; 8] = [
    "#f0bf3c", "#43e6cb", "#e8798a", "#8fb8ff", "#b9e46a", "#d39bff", "#ff9f5a", "#7fd6f0",
];
const OTHER_COLOUR: &str = "#8a93a6";

impl GraphResult {
    /// The map in Graphviz's DOT language, which `dot -Tsvg` or `dot -Tpng`
    /// draws on a dark ground. Each concept is a circle in its community's
    /// colour, sized by its passages and labelled beside it; each link is a
    /// line that grows thicker with its NPMI; and a legend names each
    /// community by its four concepts with most passages. The layout is
    /// Graphviz's force-directed `fdp` with a fixed seed, so the same map
    /// gives the same picture.
    pub fn to_dot(&self) -> String {
        let most = self
            .concepts
            .iter()
            .map(|c| c.passage_count)
            .max()
            .unwrap_or(1)
            .max(1);
        let mut dot = String::from(concat!(
            "graph map {\n",
            "  graph [layout=fdp, overlap=false, splines=true, sep=\"+12\", K=1.1, ",
            "maxiter=2000, start=7, forcelabels=true, bgcolor=\"#0a1226\", pad=0.4];\n",
            "  node [shape=circle, style=filled, fixedsize=true, label=\"\", color=\"#0f1b33\", ",
            "penwidth=1.5, fontname=\"Georgia\", fontsize=13, fontcolor=\"#f3ead0\"];\n",
            "  edge [color=\"#f3ead040\"];\n",
        ));
        for community in &self.communities {
            // A frameless cluster keeps a community together in the layout.
            let grouped = community.concepts.len() > 1;
            if grouped {
                let _ = writeln!(dot, "  subgraph cluster_{} {{ peripheries=0;", community.id);
            }
            for &id in &community.concepts {
                let concept = &self.concepts[id as usize];
                // A radius of 3.75 to 12.5 points, by the square root of the
                // concept's passages as a share of the most any concept has.
                let share = f64::from(concept.passage_count) / f64::from(most);
                let radius = 1.25 * (3.0 + 7.0 * share.sqrt());
                let _ = writeln!(
                    dot,
                    "    n{id} [width={:.3}, fillcolor=\"{}\", xlabel=\"{}\"];",
                    2.0 * radius / 72.0,
                    community_colour(concept.community),
                    dot_escape(&concept.label)
                );
            }
            if grouped {
                dot.push_str("  }\n");
            }
        }
        for link in &self.links {
            let width = 1.25 * (0.8 + 2.4 * f64::from(link.npmi));
            let _ = writeln!(dot, "  n{} -- n{} [penwidth={width:.2}];", link.a, link.b);
        }
        if !self.communities.is_empty() {
            dot.push_str(concat!(
                "  legend [shape=plaintext, style=\"\", fixedsize=false, ",
                "label=<<TABLE BORDER=\"0\" CELLSPACING=\"6\">\n"
            ));
            for community in &self.communities {
                let _ = writeln!(
                    dot,
                    "<TR><TD BGCOLOR=\"{}\" WIDTH=\"12\" HEIGHT=\"12\" FIXEDSIZE=\"TRUE\"></TD>\
                     <TD ALIGN=\"LEFT\"><FONT COLOR=\"#f3ead0\">{}</FONT></TD></TR>",
                    community_colour(community.id),
                    html_escape(&self.community_label(community))
                );
            }
            dot.push_str("</TABLE>>];\n");
        }
        dot.push_str("}\n");
        dot
    }

    /// `1 · queen, court, king, jury`: a community by its number from 1 and
    /// its four concepts with most passages.
    fn community_label(&self, community: &Community) -> String {
        let mut members: Vec<&Concept> = community
            .concepts
            .iter()
            .map(|&id| &self.concepts[id as usize])
            .collect();
        members.sort_by_key(|c| std::cmp::Reverse(c.passage_count));
        let top: Vec<&str> = members.iter().take(4).map(|c| c.label.as_str()).collect();
        format!("{} · {}", community.id + 1, top.join(", "))
    }
}

fn community_colour(community: u32) -> &'static str {
    COMMUNITY_COLOURS
        .get(community as usize)
        .copied()
        .unwrap_or(OTHER_COLOUR)
}

/// `text` inside a DOT string.
fn dot_escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/// `text` inside a DOT HTML label.
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The error `build` returns: the book has fewer than
/// `GRAPH_MIN_BOOK_PASSAGES` passages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// How many passages the book has.
    pub passages: usize,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the book has {} passages; a concept map needs at least {GRAPH_MIN_BOOK_PASSAGES}",
            self.passages
        )
    }
}

impl std::error::Error for Error {}

/// The concept map of a book's passages, which must be in reading order.
pub fn build(texts: &[&str]) -> Result<GraphResult, Error> {
    let n = texts.len();
    if n < GRAPH_MIN_BOOK_PASSAGES {
        return Err(Error { passages: n });
    }
    let selection = select(texts);
    let mut concepts = selection.concepts;
    concepts.sort_by(|x, y| {
        x.passages[0]
            .cmp(&y.passages[0])
            .then(x.label.cmp(&y.label))
    });

    let links = links(&concepts, n);
    let edges: Vec<(usize, usize, f64)> = links.iter().map(|l| (l.a, l.b, l.npmi)).collect();
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (concept, community) in louvain(concepts.len(), &edges).into_iter().enumerate() {
        if groups.len() <= community {
            groups.resize(community + 1, Vec::new());
        }
        groups[community].push(concept);
    }
    // Largest first, ties to the community the book mentions first.
    groups.sort_by(|x, y| y.len().cmp(&x.len()).then(x[0].cmp(&y[0])));
    let mut community_of = vec![0; concepts.len()];
    for (c, group) in groups.iter().enumerate() {
        for &concept in group {
            community_of[concept] = c as u32;
        }
    }

    Ok(GraphResult {
        passages: n as u32,
        topics: selection.topics,
        concepts: concepts
            .iter()
            .enumerate()
            .map(|(i, c)| Concept {
                id: i as u32,
                label: c.label.clone(),
                passage_count: c.passages.len() as u32,
                passages: c.passages.clone(),
                community: community_of[i],
            })
            .collect(),
        links: links
            .iter()
            .map(|l| Link {
                a: l.a as u32,
                b: l.b as u32,
                npmi: ((l.npmi * 1000.0).round() / 1000.0) as f32,
                passage_count: l.shared.len() as u32,
                passages: l.shared.clone(),
            })
            .collect(),
        communities: groups
            .into_iter()
            .enumerate()
            .map(|(c, group)| Community {
                id: c as u32,
                concepts: group.into_iter().map(|i| i as u32).collect(),
            })
            .collect(),
    })
}

/// A link before it is numbered for output; `shared` holds passage indices.
struct Pair {
    a: usize,
    b: usize,
    npmi: f64,
    shared: Vec<u32>,
}

/// Each concept's `GRAPH_PARTNERS` partners with the highest NPMI above
/// `GRAPH_NPMI_FLOOR`, sharing at least `GRAPH_MIN_SHARED` passages, ties to
/// the lower index. A link exists when either concept keeps the other.
fn links(concepts: &[Candidate], n: usize) -> Vec<Pair> {
    let m = concepts.len();
    let mut shared = vec![vec![Vec::new(); m]; m];
    for (a, x) in concepts.iter().enumerate() {
        for (b, y) in concepts.iter().enumerate().skip(a + 1) {
            let both = intersection(&x.passages, &y.passages);
            shared[b][a].clone_from(&both);
            shared[a][b] = both;
        }
    }
    let score = |a: usize, b: usize| {
        npmi(
            concepts[a].passages.len(),
            concepts[b].passages.len(),
            shared[a][b].len(),
            n,
        )
    };
    let mut kept = BTreeSet::new();
    for (a, with) in shared.iter().enumerate() {
        let mut partners: Vec<(f64, usize)> = (0..m)
            .filter(|&b| b != a && with[b].len() >= GRAPH_MIN_SHARED)
            .map(|b| (score(a, b), b))
            .filter(|&(v, _)| v > GRAPH_NPMI_FLOOR)
            .collect();
        partners.sort_by(|(x, i), (y, j)| y.total_cmp(x).then(i.cmp(j)));
        for (_, b) in partners.into_iter().take(GRAPH_PARTNERS) {
            kept.insert((a.min(b), a.max(b)));
        }
    }
    kept.into_iter()
        .map(|(a, b)| Pair {
            a,
            b,
            npmi: score(a, b),
            shared: shared[a][b].clone(),
        })
        .collect()
}

/// The values in both ascending lists.
fn intersection(x: &[u32], y: &[u32]) -> Vec<u32> {
    let (mut i, mut j, mut out) = (0, 0, Vec::new());
    while i < x.len() && j < y.len() {
        match x[i].cmp(&y[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                out.push(x[i]);
                i += 1;
                j += 1;
            }
        }
    }
    out
}

/// Normalized PMI of two phrases in `df_a` and `df_b` of `n` passages that
/// share `shared` of them: `ln(shared·n / (df_a·df_b)) / −ln(shared / n)`.
fn npmi(df_a: usize, df_b: usize, shared: usize, n: usize) -> f64 {
    let joint = shared as f64 / n as f64;
    (shared as f64 * n as f64 / (df_a as f64 * df_b as f64)).ln() / -joint.ln()
}

/// Steps 1 to 3 of the graph: a book's concepts, the one with the most
/// mutual information with the topics first, and how many topics there were.
#[derive(Debug, Clone, PartialEq)]
struct Selection {
    topics: u32,
    concepts: Vec<Candidate>,
}

/// The `GRAPH_CONCEPTS` candidate phrases of `texts` that best predict a
/// passage's topic, ties going to the phrase in more passages, then to the
/// lower key.
fn select(texts: &[&str]) -> Selection {
    let k = topic_count(texts.len());
    let topic_of = passage_topics(texts, k);
    let mut sizes = vec![0; k];
    for &topic in &topic_of {
        sizes[topic] += 1;
    }
    let mut scored: Vec<(f64, Candidate)> = candidates(texts)
        .into_iter()
        .map(|c| (mutual_information(&c.passages, &topic_of, &sizes), c))
        .collect();
    scored.sort_by(|(a, x), (b, y)| {
        b.total_cmp(a)
            .then(y.passages.len().cmp(&x.passages.len()))
            .then(x.key.cmp(&y.key))
    });
    scored.truncate(GRAPH_CONCEPTS);
    Selection {
        topics: k as u32,
        concepts: scored.into_iter().map(|(_, c)| c).collect(),
    }
}

/// `I(T; G)` in nats, between whether a passage is one of `occurs` and its
/// topic in `topic_of`, given how many passages each topic has.
fn mutual_information(occurs: &[u32], topic_of: &[usize], topic_sizes: &[usize]) -> f64 {
    let n = topic_of.len() as f64;
    let mut inside = vec![0; topic_sizes.len()];
    for &p in occurs {
        inside[topic_of[p as usize]] += 1;
    }
    let p_in = occurs.len() as f64 / n;
    let mut mi = 0.0;
    for (&size, &with) in topic_sizes.iter().zip(&inside) {
        let p_topic = size as f64 / n;
        for (count, p_t) in [(with, p_in), (size - with, 1.0 - p_in)] {
            if count > 0 {
                let joint = count as f64 / n;
                mi += joint * (joint / (p_t * p_topic)).ln();
            }
        }
    }
    mi
}

/// Runs the README's example as a doctest.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn npmi_is_one_for_phrases_always_together_and_zero_for_chance() {
        assert!((npmi(2, 2, 2, 10) - 1.0).abs() < 1e-12);
        assert!(npmi(4, 5, 2, 10).abs() < 1e-12);
    }

    #[test]
    fn to_dot_boxes_communities_and_draws_each_link() {
        let concept = |id: u32, label: &str, community: u32| Concept {
            id,
            label: label.to_string(),
            passage_count: 2,
            passages: vec![0, 1],
            community,
        };
        let link = |a: u32, b: u32, npmi: f32| Link {
            a,
            b,
            npmi,
            passage_count: 2,
            passages: vec![0, 1],
        };
        let map = GraphResult {
            passages: 20,
            topics: 2,
            concepts: vec![
                concept(0, "march hare", 0),
                concept(1, "hatter", 0),
                concept(2, "say \"tea\"", 1),
            ],
            links: vec![link(0, 1, 0.9), link(1, 2, 0.3)],
            communities: vec![
                Community {
                    id: 0,
                    concepts: vec![0, 1],
                },
                Community {
                    id: 1,
                    concepts: vec![2],
                },
            ],
        };

        let dot = map.to_dot();
        assert!(dot.starts_with("graph map {\n") && dot.ends_with("}\n"));
        // Only the community of two concepts is grouped.
        assert_eq!(dot.matches("subgraph cluster_").count(), 1);
        assert!(
            dot.contains(r##"n0 [width=0.347, fillcolor="#f0bf3c", xlabel="march hare"];"##),
            "{dot}"
        );
        assert!(
            dot.contains(r##"n2 [width=0.347, fillcolor="#43e6cb", xlabel="say \"tea\""];"##),
            "{dot}"
        );
        assert!(dot.contains("n0 -- n1 [penwidth=3.70];"), "{dot}");
        assert!(dot.contains("n1 -- n2 [penwidth=1.90];"), "{dot}");
        assert!(dot.contains("1 · march hare, hatter"), "{dot}");
        assert!(dot.contains("2 · say &quot;tea&quot;"), "{dot}");
    }

    #[test]
    fn a_book_shorter_than_twenty_passages_is_refused() {
        let error = build(&["one passage"; 19]).unwrap_err().to_string();
        assert!(
            error.contains("the book has 19 passages; a concept map needs at least 20"),
            "{error}"
        );
    }

    #[test]
    fn a_book_whose_phrases_are_all_too_common_has_an_empty_graph() {
        let g = build(&["the same words everywhere"; 30]).unwrap();
        assert!(g.concepts.is_empty() && g.links.is_empty() && g.communities.is_empty());
        assert_eq!(g.passages, 30);
    }

    #[test]
    fn mutual_information_matches_hand_values() {
        let topic_of = [0, 0, 1, 1];
        let sizes = [2, 2];
        let mi = mutual_information(&[0, 1], &topic_of, &sizes);
        assert!((mi - std::f64::consts::LN_2).abs() < 1e-12, "{mi}");
        assert!(mutual_information(&[0, 2], &topic_of, &sizes).abs() < 1e-12);
    }

    /// A word of letters only, different for each `j`.
    fn word(j: usize) -> String {
        let letter = |x: usize| char::from(b'a' + (x % 26) as u8);
        format!("zz{}{}", letter(j / 26), letter(j))
    }

    #[test]
    fn select_keeps_the_most_informative_concepts_up_to_the_cap() {
        // 100 passages and 80 one-word phrases, each in 3 neighbouring passages.
        let texts: Vec<String> = (0..100)
            .map(|i| {
                (0..80)
                    .filter(|j| (i + 100 - j) % 100 < 3)
                    .map(word)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .collect();
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        let s = select(&refs);
        assert_eq!(s.topics, 10);
        assert_eq!(s.concepts.len(), GRAPH_CONCEPTS);
        let topic_of = passage_topics(&refs, 10);
        let mut sizes = vec![0; 10];
        topic_of.iter().for_each(|&t| sizes[t] += 1);
        let mi: Vec<f64> = s
            .concepts
            .iter()
            .map(|c| mutual_information(&c.passages, &topic_of, &sizes))
            .collect();
        assert!(mi.windows(2).all(|w| w[0] >= w[1]), "{mi:?}");
    }
}
