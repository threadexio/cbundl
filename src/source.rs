use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::iter::{Chain, FusedIterator};
use std::path::{Path, PathBuf};
use std::slice;

use eyre::{bail, Context, Result};
use petgraph::algo::toposort;

use crate::display::display_path;
use crate::parse::source_file::SourceFile;

type Graph = petgraph::Graph<Source, (), petgraph::Directed, u32>;
type NodeIndex = petgraph::graph::NodeIndex<u32>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    Declaration,
    Implementation,
}

#[derive(Debug, Clone)]
pub struct Source {
    pub kind: SourceKind,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Sources {
    graph: Graph,
    dependencies: Vec<NodeIndex>,
}

struct SourceGraphBuilder {
    graph: Graph,
    files: HashMap<PathBuf, NodeIndex>,
}

impl SourceGraphBuilder {
    fn add_source_file(&mut self, path: PathBuf, kind: SourceKind) -> Result<NodeIndex> {
        fn realpath(path: &Path) -> Result<PathBuf> {
            fs::canonicalize(path)
                .with_context(|| format!("failed to resolve path `{}`", display_path(path)))
        }

        let raw_content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read source file `{}`", display_path(&path)))?;

        let SourceFile {
            content,
            impl_files,
            includes,
        } = SourceFile::try_parse(&raw_content)
            .with_context(|| format!("failed to parse file `{}`", display_path(&path)))?;

        let base = match path.parent() {
            None => PathBuf::from("."),
            Some(x) => x.to_path_buf(),
        };

        let real_path = realpath(&path)?;
        let me = self.graph.add_node(Source {
            kind,
            path,
            content,
        });

        self.files.insert(real_path, me);

        let includes = includes.into_iter().map(|x| (SourceKind::Declaration, x));

        let impl_files = impl_files
            .into_iter()
            .map(|x| (SourceKind::Implementation, x));

        let related_files = chain(includes, impl_files)
            .map(|(kind, x)| (kind, base.join(x)))
            .map(|(kind, x)| realpath(&x).map(|x| (kind, x)));

        for x in related_files {
            let (kind, path) = x?;

            let other = match self.files.get(&path) {
                Some(x) => *x,
                None => {
                    let other = self.add_source_file(path.clone(), kind)?;
                    self.files.insert(path, other);
                    other
                }
            };

            #[allow(clippy::single_match)]
            match (self.graph[me].kind, self.graph[other].kind) {
                // If a declaration file includes another declaration file, then we track
                // this relation to detect cyclic dependencies later.
                (SourceKind::Declaration, SourceKind::Declaration) => {
                    self.graph.add_edge(me, other, ());
                }
                // However, if an implementation file includes a declaration file, then we
                // will not keep track of this relation in order not to mess up the dependency
                // finding algorithm later. Basically if we were to track this relation,
                // then the following would not work:
                //
                // a.h:
                // // cbundl: bundle
                // #include "b.h"
                //
                // void a();
                //
                // // cbundl: impl=a.c
                //
                // a.c:
                // // cbundl: bundle
                // #include "b.h"
                //
                // void a() { b(); }
                //
                // b.h:
                // // cbundl: bundle
                // #include "a.h"
                //
                // void b();
                //
                // // cbundl: impl=b.c
                //
                // b.c:
                // // cbundl: bundle
                // #include "a.h"
                //
                // void b() { a(); }
                //
                // main.c:
                // // cbundl: bundle
                // #include "a.h"
                // // cbundl: bundle
                // #include "b.h"
                //
                // int main() { /* ... */ }
                //
                // But the above example compiles successfully because even though a circlular
                // dependency exists, the header files contain only function declarations,
                // and thus it is perfectly fine if the implementation files include them
                // in this manner. This algorithm, does run the risk of accepting invalid
                // code and creating bundles out of them. To fix this we would have to track
                // symbol dependencies (not file dependencies). Also, it is not our job to
                // perform strict checking of the code. It is then the responsibility of the
                // compiler to reject invalid code.
                _ => {}
            }
        }

        Ok(me)
    }
}

impl Sources {
    pub fn new(entry: PathBuf) -> Result<Self> {
        let mut builder = SourceGraphBuilder {
            graph: Graph::new(),
            files: HashMap::new(),
        };

        let entry = builder.add_source_file(entry, SourceKind::Implementation)?;

        let SourceGraphBuilder { graph, .. } = builder;

        let mut dependencies = match toposort(&graph, None) {
            Ok(x) => x,
            Err(_) => bail!("found circular dependency in source files"),
        };

        dependencies.sort_by(|l, r| {
            let a = &graph[*l];
            let b = &graph[*r];

            match (a.kind, b.kind) {
                (SourceKind::Declaration, SourceKind::Implementation) => Ordering::Greater,
                (SourceKind::Implementation, SourceKind::Declaration) => Ordering::Less,
                (_, _) if *l == entry => Ordering::Greater,
                (_, _) if *r == entry => Ordering::Less,
                _ => Ordering::Equal,
            }
        });
        dependencies.reverse();

        Ok(Self {
            graph,
            dependencies,
        })
    }

    pub fn dependency_order(&self) -> DependencyOrder<'_> {
        DependencyOrder {
            graph: &self.graph,
            iter: self.dependencies.iter(),
        }
    }
}

pub struct DependencyOrder<'a> {
    graph: &'a Graph,
    iter: slice::Iter<'a, NodeIndex>,
}

impl<'a> Iterator for DependencyOrder<'a> {
    type Item = &'a Source;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|idx| &self.graph[*idx])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl ExactSizeIterator for DependencyOrder<'_> {
    fn len(&self) -> usize {
        self.iter.len()
    }
}

impl FusedIterator for DependencyOrder<'_> {}

fn chain<T, A, B>(a: A, b: B) -> Chain<A, B>
where
    A: Iterator<Item = T>,
    B: Iterator<Item = T>,
{
    a.chain(b)
}
