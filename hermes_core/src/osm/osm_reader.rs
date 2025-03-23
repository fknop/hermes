use crate::geopoint::GeoPoint;
use crate::properties::car_access_parser::HIGHWAY_VALUES;
use crate::properties::property::Property;
use crate::properties::property_map::EdgePropertyMap;
use crate::properties::tag_parser::handle_way;
use osmpbf::{DenseNode, Element, ElementReader, Node, Way};
use std::{collections::HashMap, env, path::Path};

pub struct OsmNode {
    id: usize,
    pub coordinates: GeoPoint,
    tags: HashMap<String, String>,
}

pub struct OsmWay {
    id: usize,
    osm_id: usize,
    nodes: Vec<usize>,
    pub tags: HashMap<String, String>,
    properties: EdgePropertyMap,
}

impl OsmWay {
    pub fn id(&self) -> usize {
        self.id
    }

    pub fn osm_id(&self) -> usize {
        self.osm_id
    }
    pub fn tag(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(|tag| tag.as_str())
    }

    pub fn has_tag(&self, tag: &str, value: &str) -> bool {
        self.tag(tag).map_or(false, |tag_value| tag_value == value)
    }

    pub fn nodes(&self) -> &Vec<usize> {
        &self.nodes
    }

    pub fn start_node(&self) -> usize {
        self.nodes[0]
    }

    pub fn end_node(&self) -> usize {
        self.nodes[self.nodes.len() - 1]
    }

    pub fn properties(&self) -> &EdgePropertyMap {
        &self.properties
    }
    pub fn properties_mut(&mut self) -> &mut EdgePropertyMap {
        &mut self.properties
    }
}

pub struct OSMData {
    next_node_id: usize,
    next_way_id: usize,
    osm_node_ids_to_internal_id: HashMap<i64, usize>,
    osm_way_ids_to_internal_id: HashMap<i64, usize>,
    osm_node_data: Vec<OsmNode>,
    osm_ways_data: Vec<OsmWay>,
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

    pub fn nodes(&self) -> &[OsmNode] {
        &self.osm_node_data
    }

    pub fn ways(&self) -> &Vec<OsmWay> {
        &self.osm_ways_data
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
            coordinates: GeoPoint {
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
            coordinates: GeoPoint {
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

        let mut way = OsmWay {
            id: way_id,
            osm_id: way.id() as usize,
            nodes: way
                .refs()
                .filter_map(|node| self.node_id_from_osm_id(node))
                .collect(),
            tags,
            properties: EdgePropertyMap::new(),
        };

        handle_way(&mut way, Property::MaxSpeed);
        handle_way(&mut way, Property::VehicleAccess("car".to_string()));
        handle_way(&mut way, Property::OsmId);

        self.osm_ways_data.push(way);

        self.next_way_id += 1
    }

    fn node_id_from_osm_id(&self, osm_node_id: i64) -> Option<usize> {
        self.osm_node_ids_to_internal_id.get(&osm_node_id).cloned()
    }

    fn tags(&self, node_id: usize) -> Option<&HashMap<String, String>> {
        let node = self.osm_node_data.get(node_id);
        match node {
            Some(node) => Some(&node.tags),
            None => None,
        }
    }
    pub fn node(&self, id: usize) -> Option<&OsmNode> {
        self.osm_node_data.get(id)
    }

    pub fn way_geometry(&self, id: usize) -> Vec<GeoPoint> {
        let way = &self.osm_ways_data[id];
        way.nodes()
            .iter()
            .map(|node_id| self.osm_node_data[*node_id].coordinates)
            .collect()
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
    let reader = ElementReader::from_path(file_path)
        .expect(format!("Failed to read OSM file: {:?}", file_path).as_str());
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
