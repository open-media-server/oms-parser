use std::path::PathBuf;

use serde::Serialize;

#[derive(Debug, Serialize)]

pub struct Node {
    pub name: String,
    pub path: String,
    pub children: Option<Vec<Node>>,
}

pub fn parse_structure(path: &PathBuf) -> Node {
    let mut node = Node {
        name: path.file_name().unwrap().to_str().unwrap().to_string(),
        path: path.to_str().unwrap().to_string(),
        children: Some(vec![]),
    };

    parse_node(&mut node, path);
    node
}

fn parse_node(node: &mut Node, path: &PathBuf) {
    let files = path.read_dir().unwrap().filter_map(|p| p.ok());

    node.children = Some(
        files
            .map(|f| {
                let file_name = f.file_name().to_str().unwrap().to_string();
                let mut node = Node {
                    name: file_name,
                    path: f.path().to_str().unwrap().to_string(),
                    children: None,
                };

                if f.path().is_dir() {
                    parse_node(&mut node, &f.path());
                }

                node
            })
            .collect(),
    );
}
