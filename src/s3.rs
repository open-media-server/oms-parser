use regex::Regex;
use s3::{creds::Credentials, Bucket, Region};

use crate::structure::Node;

pub async fn parse_s3_structure() -> Node {
    let region = Region::Custom {
        region: "eu-central-1".to_string(),
        endpoint: "https://s3.eu-central-1.wasabisys.com".to_string(),
    };

    let bucket = Bucket::new("aerio-media", region, Credentials::default().unwrap()).unwrap();

    let res = bucket.list(String::from("anime"), None).await.unwrap();

    let regex = Regex::new(".(swf|avi|flv|mpg|rm|mov|wav|asf|3gp|mkv|rmvb|mp4)").unwrap();

    let paths = res[0]
        .contents
        .iter()
        .map(|c| c.key.as_str())
        .filter(|c| regex.is_match(c))
        .collect::<Vec<&str>>();

    paths_to_structure(paths)
}

fn paths_to_structure(paths: Vec<&str>) -> Node {
    let mut root_node = Node {
        name: String::from(""),
        path: String::from(""),
        children: None,
    };

    for path in paths {
        let mut curr_node = &mut root_node;

        let parts = path.split("/");
        for part in parts {
            if curr_node.children.is_none() {
                curr_node.children = Some(vec![]);
            }

            let children = curr_node.children.as_mut().unwrap();

            let node = match children.iter().position(|n| n.name == part) {
                Some(i) => &mut children[i],
                None => {
                    let node = Node {
                        name: part.to_string(),
                        path: format!("{}/{}", curr_node.path, part),
                        children: None,
                    };
                    children.push(node);
                    children.last_mut().unwrap()
                }
            };

            curr_node = node;
        }
    }

    root_node
}
