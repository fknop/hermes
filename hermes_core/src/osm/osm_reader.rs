use crate::latlng::LatLng;
use osmpbf::{DenseNode, Element, ElementReader, Node, Way};
use std::collections::HashMap;

pub struct OsmNode {
    id: usize,
    pub coordinates: LatLng,
    tags: HashMap<String, String>,
}

pub struct OsmWay {
    id: usize,
    pub nodes: Vec<usize>,
    pub tags: HashMap<String, String>,
}

impl OsmWay {
    pub fn get_tag(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(|tag| tag.as_str())
    }

    pub fn has_tag(&self, tag: &str, value: &str) -> bool {
        self.get_tag(tag)
            .map_or(false, |tag_value| tag_value == value)
    }
}

pub struct OSMData {
    next_node_id: usize,
    next_way_id: usize,
    osm_node_ids_to_internal_id: HashMap<i64, usize>,
    osm_way_ids_to_internal_id: HashMap<i64, usize>,
    pub osm_node_data: Vec<OsmNode>,
    pub osm_ways_data: Vec<OsmWay>,
}

impl OSMData {
    fn new() -> Self {
        OSMData {
            next_node_id: 0,
            next_way_id: 0,
            osm_node_ids_to_internal_id: HashMap::new(),
            osm_way_ids_to_internal_id: HashMap::new(),
            osm_node_data: Vec::new(),
            osm_ways_data: Vec::new(),
        }
    }

    fn add_node(&mut self, node: &Node) {
        let node_id = self.next_node_id;
        self.osm_node_ids_to_internal_id.insert(node.id(), node_id);

        let tags: HashMap<String, String> = node
            .tags()
            .into_iter()
            .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
            .collect();

        self.osm_node_data.push(OsmNode {
            id: node_id,
            coordinates: LatLng {
                lat: node.lat(),
                lng: node.lon(),
            },
            tags,
        });
        self.next_node_id += 1;
    }

    fn add_dense_node(&mut self, node: &DenseNode) {
        let node_id = self.next_node_id;
        self.osm_node_ids_to_internal_id.insert(node.id(), node_id);

        let tags: HashMap<String, String> = node
            .tags()
            .into_iter()
            .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
            .collect();

        self.osm_node_data.push(OsmNode {
            id: node_id,
            coordinates: LatLng {
                lat: node.lat(),
                lng: node.lon(),
            },
            tags,
        });
        self.next_node_id += 1;
    }

    fn add_way(&mut self, way: &Way) {
        let tags: HashMap<String, String> = way
            .tags()
            .into_iter()
            .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
            .collect();

        if !tags.contains_key("highway") {
            return;
        }

        let way_id = self.next_way_id;
        self.osm_way_ids_to_internal_id.insert(way.id(), way_id);

        self.osm_ways_data.push(OsmWay {
            id: way_id,
            nodes: way
                .refs()
                .filter_map(|node| self.get_node_id_from_osm_id(node))
                .collect(),
            tags,
        });

        self.next_way_id += 1
    }

    fn get_node_id_from_osm_id(&self, osm_node_id: i64) -> Option<usize> {
        self.osm_node_ids_to_internal_id.get(&osm_node_id).cloned()
    }

    fn get_tags(&self, node_id: usize) -> Option<&HashMap<String, String>> {
        let node = self.osm_node_data.get(node_id);
        match node {
            Some(node) => Some(&node.tags),
            None => None,
        }
    }

    pub fn get_nodes(&self) -> &Vec<OsmNode> {
        &self.osm_node_data
    }

    pub fn get_node(&self, id: usize) -> Option<&OsmNode> {
        self.osm_node_data.get(id)
    }

    pub fn get_ways(&self) -> &Vec<OsmWay> {
        &self.osm_ways_data
    }
}

fn accept_way(way: &Way) -> bool {
    if way.refs().len() < 2 {
        return false;
    }

    if way.tags().len() == 0 {
        return false;
    }

    true
}

pub fn parse_osm_file(file_path: &str) -> Box<OSMData> {
    let reader = ElementReader::from_path(file_path).expect("Failed to read OSM file");
    let mut node_count = 0_i64;
    let mut way_count = 0_i64;

    let mut osm_data = Box::new(OSMData::new());

    reader
        .for_each(|element| {
            match element {
                Element::Relation(_) => {
                    // Process relation data
                    // println!("Relation ID: {}", relation.id());
                }
                Element::DenseNode(node) => {
                    osm_data.add_dense_node(&node);
                    node_count += 1;
                    if node_count % 10000 == 0 {
                        println!("Processed {} nodes", node_count);
                    }
                }
                Element::Node(node) => {
                    osm_data.add_node(&node);
                    node_count += 1;
                    if node_count % 10000 == 0 {
                        println!("Processed {} nodes", node_count);
                    }
                }
                Element::Way(way) if accept_way(&way) => {
                    // Process way data
                    osm_data.add_way(&way);
                    way_count += 1;
                    if way_count % 10000 == 0 {
                        println!("Processed {} ways", way_count);
                    }
                }
                _ => {}
            }
        })
        .expect("Failed to parse OSM file");

    osm_data
}
