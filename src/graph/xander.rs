use bevy::prelude::*;
use std::collections::HashSet;
use rand::seq::IteratorRandom;

use crate::graph::grid_transform::GridTransform;
use crate::graph::node::NodeBundle;
use crate::graph::node::Node;
use crate::graph::connection::Connection;

#[derive(Event, Default)]
pub struct XanderEvent;

pub struct XanderPlugin;

impl Plugin for XanderPlugin { 
    fn build(&self, app: &mut App) {
        app
            .add_event::<XanderEvent>()
            .add_systems(Update, xander);
    }
}

fn xander(
    mut commands: Commands,
    mut event: EventReader<XanderEvent>,
    query: Query<&GridTransform, With<Node>>,
    connections_query: Query<&Connection>,
) {
    let mut rng = rand::thread_rng();
    for _ in event.read() {
        // info!("Xander Event processing!");

        let mut nodes = HashSet::new();
        for pos in query.iter() {
            nodes.insert(*pos);
        }
        // info!("nodes: {}", nodes.len());

        let connections: Vec<Connection> = connections_query.iter().cloned().collect();

        let mut neighbor_node_pairs = Vec::new();
        for node in &nodes {
            for neighbor_position in node.neighbors_ordinals(connections.clone()) {
                if nodes.contains(&neighbor_position) {
                    neighbor_node_pairs.push((node, neighbor_position));
                }
            }
        }
        // info!("neighbor_node_pairs: {}", neighbor_node_pairs.len());

        let mut loop_candidates = Vec::new();
        for (a, b) in neighbor_node_pairs {
            let mut a_empty_neighbors = Vec::new();
            for pos in a.neighbors_ordinals(connections.clone()) {
                if !nodes.contains(&pos) {
                    a_empty_neighbors.push(pos);
                }
            }

            let mut b_empty_neighbors = HashSet::new();
            for pos in b.neighbors_ordinals(connections.clone()) {
                if !nodes.contains(&pos) {
                    b_empty_neighbors.insert(pos);
                }
            }

            for an in a_empty_neighbors {
                for an2 in an.neighbors_ordinals(connections.clone()) {
                    if b_empty_neighbors.contains(&an2) {
                        let conn_a_an = Connection::new(*a, an);
                        let conn_an2_b = Connection::new(an2, b);

                        if !conn_a_an.crosses(&conn_an2_b) {
                            loop_candidates.push((a, an, an2, b));
                        }
                    }
                }
            }
        }
        // info!("loop_candidates: {}", loop_candidates.len());

        if let Some((a, b, c, d)) = loop_candidates.into_iter().choose(&mut rng) {
            commands.spawn(NodeBundle {
                position: b,
                ..Default::default()
            });
            commands.spawn(NodeBundle {
                position: c,
                ..Default::default()
            });

            let connection_sets = vec![
                vec![ (*a, b), (b, c), (c, d) ],
                vec![ (*a, b), (b, c), (c, d) ],
                vec![ (*a, b), (b, c), (c, d) ],
                vec![ (b, c), (c, d) ],
                vec![ (*a, b), (c, d) ],
                vec![ (*a, b), (b, c) ],
            ];

            if let Some(con_set) = connection_sets.into_iter().choose(&mut rng) {
                for (a, b) in con_set {
                    commands.spawn(Connection {
                        a,
                        b,
                    });
                }
            } 
        }
    }
}

