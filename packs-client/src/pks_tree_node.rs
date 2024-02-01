use crate::pks::PathViolations;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, PartialEq)]
pub struct PksTreeNode {
    pub path: String,
    pub node_name: String,
    pub has_package_definition: bool,
    pub children: Option<Vec<PksTreeNode>>,
    pub in_violation_count: usize,
    pub out_violation_count: usize,
}

pub struct PksTreeBuilder {
    pack_names: Vec<String>,
    violation_counts: Arc<HashMap<String, Arc<PathViolations>>>,
    pub children: Vec<PksTreeNode>,
}

impl PksTreeBuilder {
    pub fn new(
        pack_names: Vec<String>,
        violation_counts: Arc<HashMap<String, Arc<PathViolations>>>,
    ) -> Self {
        let mut builder = Self {
            pack_names,
            violation_counts,
            children: Vec::new(),
        };
        builder.build();
        builder
    }

    fn build(&mut self) {
        for pack_name in self.pack_names.clone() {
            let pack_name_parts = pack_name_to_node_names(&pack_name);
            let mut pack_name_parts = pack_name_parts.into_iter().peekable();
            while let Some((pack_name, name)) = pack_name_parts.next() {
                self.find_or_create_node(pack_name, name, pack_name_parts.peek().is_none());
            }
        }
    }
    fn find_or_create_node(&mut self, pack_name: String, node_name: String, leaf: bool) {
        let dirs = pack_name_to_node_names(&pack_name);
        let mut dir_parts = dirs.into_iter().peekable();
        let mut current_nodes = &mut self.children;
        while let Some((pack_name, _)) = dir_parts.next() {
            if dir_parts.peek().is_some() {
                let node: &mut PksTreeNode = current_nodes
                    .iter_mut()
                    .find(|node| node.path == pack_name)
                    .unwrap();
                if node.children.is_none() {
                    node.children = Some(Vec::new());
                }
                let children: &mut Vec<PksTreeNode> = node.children.as_mut().unwrap();
                current_nodes = children;
            }
        }
        if let Some(found) = current_nodes.iter_mut().find(|node| node.path == pack_name) {
            if !found.has_package_definition && leaf {
                found.has_package_definition = true;
            }
            return;
        }

        let children = if leaf { None } else { Some(Vec::new()) };
        let vc = self.violation_counts.get(&pack_name);
        let (in_violation_count, out_violation_count) = match vc {
            Some(vc) => (
                vc.uncontained_in_violations_count,
                vc.uncontained_out_violations_count,
            ),
            None => (0, 0),
        };
        let new_node = PksTreeNode {
            path: pack_name,
            node_name,
            children,
            in_violation_count,
            out_violation_count,
            has_package_definition: leaf,
        };

        current_nodes.push(new_node);
    }
}

pub(crate) fn pack_name_to_node_names(
    pack_name: &str,
) -> Vec<(String /* pack_name */, String /* name */)> {
    let pack_name = prepend_dot_to_path(pack_name);
    let mut pack_name_parts: Vec<&str> = pack_name.split('/').collect();
    let mut node_names = Vec::new();
    let mut pack_name = pack_name.to_owned();
    while let Some(part) = pack_name_parts.pop() {
        node_names.insert(0, (pack_name.clone(), part.to_string()));
        pack_name = pack_name_parts.join("/");
    }
    node_names
}

pub(crate) fn prepend_dot_to_path(path: &str) -> String {
    if path.starts_with('.') {
        path.to_owned()
    } else {
        format!("./{}", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_pks_tree_data() {
        let pack_names = vec![
            String::from("."),
            String::from("packs/product_services/payroll/show_me_the_money"),
        ];
        let result = PksTreeBuilder::new(pack_names, Arc::new(HashMap::new())).children;
        let expected = vec![PksTreeNode {
            path: String::from("."),
            node_name: String::from("."),
            has_package_definition: true,
            in_violation_count: 0,
            out_violation_count: 0,
            children: Some(vec![PksTreeNode {
                path: String::from("./packs"),
                node_name: String::from("packs"),
                has_package_definition: false,
                in_violation_count: 0,
                out_violation_count: 0,
                children: Some(vec![PksTreeNode {
                    path: String::from("./packs/product_services"),
                    node_name: String::from("product_services"),
                    has_package_definition: false,
                    in_violation_count: 0,
                    out_violation_count: 0,
                    children: Some(vec![PksTreeNode {
                        path: String::from("./packs/product_services/payroll"),
                        node_name: String::from("payroll"),
                        has_package_definition: false,
                        in_violation_count: 0,
                        out_violation_count: 0,
                        children: Some(vec![PksTreeNode {
                            path: String::from(
                                "./packs/product_services/payroll/show_me_the_money",
                            ),
                            node_name: String::from("show_me_the_money"),
                            has_package_definition: true,
                            in_violation_count: 0,
                            out_violation_count: 0,
                            children: None,
                        }]),
                    }]),
                }]),
            }]),
        }];
        assert_eq!(result, expected);
    }

    #[test]
    fn test_pack_name_to_node_names() {
        assert_eq!(
            pack_name_to_node_names(&String::from(".")),
            vec![(String::from("."), String::from("."))]
        );
        assert_eq!(
            pack_name_to_node_names(&String::from("packs/foo")),
            vec![
                (String::from("."), String::from(".")),
                (String::from("./packs"), String::from("packs")),
                (String::from("./packs/foo"), String::from("foo")),
            ]
        );
        assert_eq!(
            pack_name_to_node_names(&String::from("packs/product_services/bar")),
            vec![
                (String::from("."), String::from(".")),
                (String::from("./packs"), String::from("packs")),
                (
                    String::from("./packs/product_services"),
                    String::from("product_services")
                ),
                (
                    String::from("./packs/product_services/bar"),
                    String::from("bar")
                ),
            ]
        );
        assert_eq!(
            pack_name_to_node_names(&String::from("packs/payroll/payroll")),
            vec![
                (String::from("."), String::from(".")),
                (String::from("./packs"), String::from("packs")),
                (String::from("./packs/payroll"), String::from("payroll")),
                (
                    String::from("./packs/payroll/payroll"),
                    String::from("payroll")
                ),
            ]
        );
    }
}
