use crate::geopoint::GeoPoint;
use crate::properties::property::Property;
use crate::properties::property_map::EdgePropertyMap;
use crate::properties::tag_parser::handle_way;
use osmio::{
    Node, OSMObj, OSMObjBase, OSMReader, Way,
    obj_types::{StringOSMObj, StringWay},
};
use osmpbf::{Element, ElementReader};
use std::{collections::HashMap, fs::File, io::BufReader};

struct OsmNode {
    coordinates: GeoPoint,
    tags: HashMap<String, String>,
}

pub struct OsmWay {
    osm_id: usize,
    tags: HashMap<String, String>,
}

impl OsmWay {
    pub fn osm_id(&self) -> usize {
        self.osm_id
    }
    pub fn tag(&self, tag: &str) -> Option<&str> {
        self.tags.get(tag).map(|tag| tag.as_str())
    }

    pub fn has_tag(&self, tag: &str, value: &str) -> bool {
        self.tag(tag) == Some(value)
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
enum OsmNodeType {
    /// Start or end of a line
    End,
    /// Junction nodes nodes that are present in two different ways but are not the start or ending node
    /// We need to store it so we can split the way later on
    Junction,
    /// Geometry nodes are only used in ways to represent to geometry
    Geometry,
}

pub struct OsmWaySegment {
    pub start_node: usize,
    pub end_node: usize,
    pub geometry: Vec<GeoPoint>,
    pub properties: EdgePropertyMap,
}

type OsmioReader = osmio::pbf::PBFReader<BufReader<File>>;

#[derive(Default)]
pub struct OsmReader {
    accepted_ways: usize,
    processed_segments: usize,
    routing_nodes: Vec<OsmNode>,
    geometry_nodes: Vec<OsmNode>,
    osm_node_id_to_node_type: HashMap<i64, OsmNodeType>,
    osm_node_id_to_node_id: HashMap<i64, usize>,
}

impl OsmReader {
    fn update_node_type(&mut self, osm_node_id: i64, node_type: OsmNodeType) {
        let new_node_type = match self.osm_node_id_to_node_type.get(&osm_node_id) {
            // If already a junction, do nothing
            Some(OsmNodeType::Junction) => OsmNodeType::Junction,

            Some(OsmNodeType::End) if node_type == OsmNodeType::Geometry => OsmNodeType::Junction,

            Some(OsmNodeType::Geometry) => OsmNodeType::Junction,

            _ => node_type,
        };

        self.osm_node_id_to_node_type
            .insert(osm_node_id, new_node_type);
    }

    fn generate_next_routing_node_id(&self) -> usize {
        self.routing_nodes.len()
    }

    fn generate_next_geometry_node_id(&self) -> usize {
        self.geometry_nodes.len()
    }

    fn accept_way(way: &StringWay) -> bool {
        if way.num_nodes() < 2 {
            return false;
        }

        if way.num_tags() == 0 {
            return false;
        }

        if !way.has_tag("highway") {
            return false;
        }

        true
    }

    /// First pass reads the ways, stores each node
    fn handle_element_first_pass(&mut self, reader: &mut OsmioReader) {
        reader
            .ways()
            .filter(|way| OsmReader::accept_way(way))
            .for_each(|way| {
                let way_node_count = way.num_nodes();

                for (index, node) in way.nodes().iter().enumerate() {
                    let is_start_or_end = index == 0 || index == way_node_count - 1;

                    self.update_node_type(
                        *node,
                        if is_start_or_end {
                            OsmNodeType::End
                        } else {
                            OsmNodeType::Geometry
                        },
                    );
                }

                self.accepted_ways += 1;

                if self.accepted_ways % 50000 == 0 {
                    println!("Preprocessed {} ways", self.accepted_ways);
                }
            });

        println!("Preprocessed {} ways", self.accepted_ways);

        // match element {
        //     Element::Way(way) if OsmReader::accept_way(&way) => {
        //         let way_node_count = way.refs().len();

        //         for (index, node) in way.refs().enumerate() {
        //             let is_start_or_end = index == 0 || index == way_node_count - 1;

        //             if node == 4597819068 {
        //                 println!("Add node {}", node);
        //             }
        //             self.update_node_type(
        //                 node,
        //                 if is_start_or_end {
        //                     OsmNodeType::End
        //                 } else {
        //                     OsmNodeType::Geometry
        //                 },
        //             );
        //         }

        //         self.accepted_ways += 1;

        //         if self.accepted_ways % 10000 == 0 {
        //             println!("Preprocessed {} ways", self.accepted_ways);
        //         }
        //     }
        //     _ => {}
        // }
    }

    fn handle_element_second_pass<F>(&mut self, reader: &mut OsmioReader, mut handle_edge: F)
    where
        F: FnMut(OsmWaySegment) -> (),
    {
        reader
            .objects()
            .filter(|element| element.is_node() || element.is_way())
            .for_each(|element| match element {
                StringOSMObj::Node(node) => {
                    let lat_lon = node.lat_lon_f64();
                    if let Some((lat, lon)) = lat_lon {
                        self.add_node(
                            node.id(),
                            GeoPoint { lat, lon },
                            node.tags()
                                .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
                                .collect(),
                        );
                    }
                }
                StringOSMObj::Way(raw_way) if OsmReader::accept_way(&raw_way) => {
                    let mut way = OsmWay {
                        osm_id: raw_way.id() as usize,
                        tags: raw_way
                            .tags()
                            .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
                            .collect(),
                    };

                    let mut properties = EdgePropertyMap::new();

                    // TODO: move somewhere else
                    handle_way(&mut way, &mut properties, Property::MaxSpeed);
                    handle_way(
                        &mut way,
                        &mut properties,
                        Property::VehicleAccess("car".to_string()),
                    );
                    handle_way(
                        &mut way,
                        &mut properties,
                        Property::AverageSpeed("car".to_string()),
                    );
                    handle_way(&mut way, &mut properties, Property::OsmId);

                    let nodes = raw_way.nodes();

                    let segments = self.split_way(&nodes);

                    for segment in segments {
                        if !self.is_routing_node(segment[0]) {
                            println!(
                                "FIRST: node {} is not a routing node, geometry: {}",
                                segment[0],
                                self.is_geometry_node(segment[0])
                            );
                        }

                        if !self.is_routing_node(segment[segment.len() - 1]) {
                            println!(
                                "LAST: node {} is not a routing node geometry: {}",
                                segment[segment.len() - 1],
                                self.is_geometry_node(segment[segment.len() - 1])
                            );
                        }

                        let start_node = self.osm_node_id_to_node_id[&segment[0]];
                        let end_node = self.osm_node_id_to_node_id[&segment[segment.len() - 1]];

                        let geometry: Vec<GeoPoint> = segment
                            .iter()
                            .filter(|osm_node_id| {
                                self.osm_node_id_to_node_id.contains_key(osm_node_id)
                            })
                            .map(|osm_node_id| {
                                let node_id = self.osm_node_id_to_node_id[osm_node_id];
                                if self.is_routing_node(*osm_node_id) {
                                    self.routing_nodes[node_id].coordinates
                                } else if self.is_geometry_node(*osm_node_id) {
                                    self.geometry_nodes[node_id].coordinates
                                } else {
                                    panic!(
                                        "Unknown node type osm_node_id={}, node_id={}",
                                        osm_node_id, node_id
                                    );
                                }
                            })
                            .collect();

                        handle_edge(OsmWaySegment {
                            start_node,
                            end_node,
                            geometry,
                            properties: properties.clone(),
                        });

                        self.processed_segments += 1;

                        if self.processed_segments % 50000 == 0 {
                            println!("Processed {} segments", self.processed_segments)
                        }
                    }

                    // TODO: Split ways at JUNCTION nodes
                    //
                }
                _ => (),
            });

        // match element {
        //     Element::Relation(_) => {
        //         // Process relation data
        //         // println!("Relation ID: {}", relation.id());
        //     }
        //     Element::DenseNode(node) => {
        //         self.add_node(
        //             node.id(),
        //             GeoPoint {
        //                 lat: node.lat(),
        //                 lon: node.lon(),
        //             },
        //             node.tags()
        //                 .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
        //                 .collect(),
        //         );
        //     }
        //     Element::Node(node) => self.add_node(
        //         node.id(),
        //         GeoPoint {
        //             lat: node.lat(),
        //             lon: node.lon(),
        //         },
        //         node.tags()
        //             .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
        //             .collect(),
        //     ),
        //     Element::Way(raw_way) if OsmReader::accept_way(&raw_way) => {
        //         let mut way = OsmWay {
        //             osm_id: raw_way.id() as usize,
        //             tags: raw_way
        //                 .tags()
        //                 .map(|tag| (tag.0.to_owned(), tag.1.to_owned()))
        //                 .collect(),
        //         };

        //         let mut properties = EdgePropertyMap::new();

        //         // TODO: move somewhere else
        //         handle_way(&mut way, &mut properties, Property::MaxSpeed);
        //         handle_way(
        //             &mut way,
        //             &mut properties,
        //             Property::VehicleAccess("car".to_string()),
        //         );
        //         handle_way(&mut way, &mut properties, Property::OsmId);

        //         let nodes: Vec<i64> = raw_way
        //             .refs()
        //             .filter(|osm_id| self.osm_node_id_to_node_id.contains_key(osm_id))
        //             .collect();
        //         let segments = self.split_way(&nodes);

        //         for segment in segments {
        //             let start_node = self.osm_node_id_to_node_id[&segment[0]];
        //             let end_node = self.osm_node_id_to_node_id[&segment[segment.len() - 1]];

        //             let geometry: Vec<GeoPoint> = segment
        //                 .iter()
        //                 .map(|osm_node_id| {
        //                     let node_id = self.osm_node_id_to_node_id[osm_node_id];
        //                     if self.is_routing_node(*osm_node_id) {
        //                         self.routing_nodes[node_id].coordinates
        //                     } else if self.is_geometry_node(*osm_node_id) {
        //                         self.geometry_nodes[node_id].coordinates
        //                     } else {
        //                         panic!(
        //                             "Unknown node type osm_node_id={}, node_id={}",
        //                             osm_node_id, node_id
        //                         );
        //                     }
        //                 })
        //                 .collect();

        //             handle_edge(OsmWaySegment {
        //                 start_node,
        //                 end_node,
        //                 geometry,
        //                 properties: properties.clone(),
        //             });

        //             self.processed_segments += 1;

        //             if self.processed_segments % 10000 == 0 {
        //                 println!("Processed {} segments", self.processed_segments)
        //             }
        //         }

        //         // TODO: Split ways at JUNCTION nodes
        //         //
        //     }
        //     _ => {}
        // }
    }

    fn split_way<'a>(&self, node_osm_ids: &'a [i64]) -> Vec<&'a [i64]> {
        let mut start: usize = 0;

        let mut segments: Vec<&'a [i64]> = Vec::new();

        for (index, osm_id) in node_osm_ids.iter().enumerate() {
            let node_type = self
                .osm_node_id_to_node_type
                .get(osm_id)
                .unwrap_or_else(|| {
                    panic!(
                        "Node {} in way is missing from osm_node_id_to_node_type",
                        osm_id
                    )
                });

            if *node_type == OsmNodeType::Junction && index != start {
                let segment = &node_osm_ids[start..=index];
                segments.push(segment);
                start = index;
            }
        }

        if start < node_osm_ids.len() - 1 {
            segments.push(&node_osm_ids[start..]);
        }

        segments
    }

    fn add_node(&mut self, osm_id: i64, coordinates: GeoPoint, tags: HashMap<String, String>) {
        let node_type = self.osm_node_id_to_node_type.get(&osm_id);

        if node_type.is_none() {
            return;
        }

        match node_type {
            Some(OsmNodeType::End) => self.add_routing_node(osm_id, coordinates, tags),
            Some(OsmNodeType::Junction) => self.add_routing_node(osm_id, coordinates, tags),
            Some(OsmNodeType::Geometry) => self.add_geometry_node(osm_id, coordinates, tags),
            None => (),
        }

        let node_count = self.routing_nodes.len() + self.geometry_nodes.len();

        if node_count % 100000 == 0 {
            println!("Processed {} nodes", node_count);
        }
    }

    #[inline(always)]
    fn add_routing_node(
        &mut self,
        osm_id: i64,
        coordinates: GeoPoint,
        tags: HashMap<String, String>,
    ) {
        let node_id = self.generate_next_routing_node_id();
        self.osm_node_id_to_node_id.insert(osm_id, node_id);
        self.routing_nodes.push(OsmNode { tags, coordinates })
    }

    #[inline(always)]
    fn add_geometry_node(
        &mut self,
        osm_id: i64,
        coordinates: GeoPoint,
        tags: HashMap<String, String>,
    ) {
        let node_id = self.generate_next_geometry_node_id();
        self.osm_node_id_to_node_id.insert(osm_id, node_id);
        self.geometry_nodes.push(OsmNode { tags, coordinates })
    }

    fn is_routing_node(&self, osm_id: i64) -> bool {
        match self.osm_node_id_to_node_type.get(&osm_id) {
            Some(OsmNodeType::End) => true,
            Some(OsmNodeType::Junction) => true,
            Some(OsmNodeType::Geometry) => false,
            None => false,
        }
    }

    fn is_geometry_node(&self, osm_id: i64) -> bool {
        match self.osm_node_id_to_node_type.get(&osm_id) {
            Some(OsmNodeType::Geometry) => true,
            _ => false,
        }
    }

    fn read_osm<F: for<'a> FnMut(&mut osmio::pbf::PBFReader<BufReader<File>>)>(
        file_path: &str,
        mut handler: F,
    ) {
        let mut reader = osmio::read_pbf(file_path)
            .expect(format!("Failed to read OSM file: {:?}", file_path).as_str());

        handler(&mut reader);

        // let reader = ElementReader::from_path(file_path)
        //     .expect(format!("Failed to read OSM file: {:?}", file_path).as_str());

        // reader.for_each(handler).expect("Failed to parse OSM file");
    }

    pub fn parse_osm_file<F>(&mut self, file_path: &str, mut handle_edge: F)
    where
        F: FnMut(OsmWaySegment) -> (),
    {
        println!("Starting first OSM parsing pass");
        OsmReader::read_osm(file_path, |reader| self.handle_element_first_pass(reader));
        println!("Starting second OSM parsing pass");
        OsmReader::read_osm(file_path, |reader| {
            self.handle_element_second_pass(reader, &mut handle_edge)
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_osm_reader() -> OsmReader {
        OsmReader::default()
    }

    fn setup_node_types(reader: &mut OsmReader, node_types: Vec<(i64, OsmNodeType)>) {
        for (id, node_type) in node_types {
            reader.osm_node_id_to_node_type.insert(id, node_type);
        }
    }

    #[test]
    fn test_split_way_no_junctions() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::End),
                (2, OsmNodeType::Geometry),
                (3, OsmNodeType::Geometry),
                (4, OsmNodeType::End),
            ],
        );

        let way = vec![1, 2, 3, 4];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], &[1, 2, 3, 4]);
    }

    #[test]
    fn test_split_way_one_junction() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::End),
                (2, OsmNodeType::Junction),
                (3, OsmNodeType::End),
            ],
        );

        let way = vec![1, 2, 3];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0], &[1, 2]);
        assert_eq!(segments[1], &[2, 3]);
    }

    #[test]
    fn test_split_way_multiple_junctions() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::End),
                (2, OsmNodeType::Geometry),
                (3, OsmNodeType::Junction),
                (4, OsmNodeType::Geometry),
                (5, OsmNodeType::Junction),
                (6, OsmNodeType::End),
            ],
        );

        let way = vec![1, 2, 3, 4, 5, 6];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], &[1, 2, 3]);
        assert_eq!(segments[1], &[3, 4, 5]);
        assert_eq!(segments[2], &[5, 6]);
    }

    #[test]
    fn test_split_way_consecutive_junctions() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::End),
                (2, OsmNodeType::Junction),
                (3, OsmNodeType::Junction),
                (4, OsmNodeType::End),
            ],
        );

        let way = vec![1, 2, 3, 4];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 3);
        assert_eq!(segments[0], &[1, 2]);
        assert_eq!(segments[1], &[2, 3]);
        assert_eq!(segments[2], &[3, 4]);
    }

    #[test]
    fn test_split_way_junction_at_ends() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::Junction),
                (2, OsmNodeType::Geometry),
                (3, OsmNodeType::Junction),
            ],
        );

        let way = vec![1, 2, 3];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], &[1, 2, 3]);
    }

    #[test]
    fn test_split_way_junction_at_one_end() {
        let mut reader = create_test_osm_reader();
        setup_node_types(
            &mut reader,
            vec![
                (1, OsmNodeType::End),
                (2, OsmNodeType::Geometry),
                (3, OsmNodeType::Junction),
            ],
        );

        let way = vec![1, 2, 3];
        let segments = reader.split_way(&way);

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0], &[1, 2, 3]);
    }
}
