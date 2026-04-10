use std::path::PathBuf;

pub struct FileNode {
    pub path: PathBuf,
    pub is_dir: bool,
    pub expanded: bool,
    pub children: Vec<usize>,
}

pub struct FileTree {
    pub nodes: Vec<FileNode>,
    pub root: usize,
    pub selected: usize,
    pub visible: bool,
    pub focused: bool,
}

impl FileTree {
    pub fn new(path: PathBuf) -> Self {
        let is_dir = path.is_dir();
        let root_node = FileNode {
            path,
            is_dir,
            expanded: false,
            children: vec![],
        };

        let mut tree = Self {
            nodes: vec![root_node],
            root: 0,
            selected: 0,
            visible: true,
            focused: false,
        };

        tree.expand(0);
        tree
    }

    pub fn expand(&mut self, node_idx: usize) {
        let node_path = self.nodes[node_idx].path.clone();
        let is_node_dir = self.nodes[node_idx].is_dir;

        if !is_node_dir || self.nodes[node_idx].expanded {
            return;
        }

        if self.nodes[node_idx].children.is_empty() {
            use ignore::WalkBuilder;
            let mut new_children = vec![];

            let walker = WalkBuilder::new(&node_path)
                .max_depth(Some(1))
                .hidden(false) // showing hidden files is usually preferred in trees unless specified
                .git_ignore(true)
                .build();

            let mut entries: Vec<_> = walker.filter_map(|e| e.ok()).collect();

            // Sort: folders first, then files
            entries.sort_by_key(|e| {
                let is_file = e.path().is_file();
                (is_file, e.path().to_path_buf())
            });

            for e in entries {
                let path = e.path().to_path_buf();
                if path == node_path {
                    continue;
                }

                let child_node = FileNode {
                    path,
                    is_dir: e.path().is_dir(),
                    expanded: false,
                    children: vec![],
                };
                self.nodes.push(child_node);
                new_children.push(self.nodes.len() - 1);
            }

            self.nodes[node_idx].children = new_children;
        }
        self.nodes[node_idx].expanded = true;
    }

    pub fn collapse(&mut self, node_idx: usize) {
        self.nodes[node_idx].expanded = false;
    }

    pub fn toggle(&mut self, node_idx: usize) {
        if self.nodes[node_idx].expanded {
            self.collapse(node_idx);
        } else {
            self.expand(node_idx);
        }
    }

    pub fn flatten(&self) -> Vec<(usize, usize)> {
        // (node_idx, depth)
        let mut result = vec![];
        for &child in &self.nodes[self.root].children {
            self.flatten_recursive(child, 0, &mut result);
        }
        result
    }

    fn flatten_recursive(&self, node_idx: usize, depth: usize, result: &mut Vec<(usize, usize)>) {
        result.push((node_idx, depth));
        let node = &self.nodes[node_idx];
        if node.expanded {
            for &child in &node.children {
                self.flatten_recursive(child, depth + 1, result);
            }
        }
    }
}
