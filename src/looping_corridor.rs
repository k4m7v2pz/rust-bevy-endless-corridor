//! 循环走廊地图生成系统
//! 创建环形布局的地图结构，增强迷宫感和探索深度

use rand::Rng;
use std::collections::HashMap;

// ---------- 数据结构 ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorridorType {
    Straight,
    Curved,
    Branching,
    Looping,
    Spiral,
}

#[derive(Debug, Clone)]
pub struct MapNode {
    pub node_id: String,
    pub x: f32,
    pub y: f32,
    pub connections: Vec<String>,
    pub room_type: String,
    pub visited: bool,
    pub discovery_order: u32,
}

impl MapNode {
    pub fn new(node_id: &str, x: f32, y: f32) -> Self {
        Self {
            node_id: node_id.to_string(),
            x,
            y,
            connections: Vec::new(),
            room_type: "corridor".to_string(),
            visited: false,
            discovery_order: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CorridorSegment {
    pub start_node: String,
    pub end_node: String,
    pub segment_type: CorridorType,
    pub length: f32,
    pub curvature: f32,
}

// ---------- 生成器 ----------

pub struct LoopingCorridorGenerator {
    pub width: f32,
    pub height: f32,
    pub nodes: HashMap<String, MapNode>,
    pub corridors: Vec<CorridorSegment>,
    pub loop_points: Vec<(f32, f32)>,
    discovery_counter: u32,
    pub min_segment_length: f32,
    pub max_segment_length: f32,
    pub branch_probability: f32,
    pub loop_probability: f32,
    pub spiral_turns: f32,
}

impl LoopingCorridorGenerator {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            nodes: HashMap::new(),
            corridors: Vec::new(),
            loop_points: Vec::new(),
            discovery_counter: 0,
            min_segment_length: 80.0,
            max_segment_length: 150.0,
            branch_probability: 0.3,
            loop_probability: 0.4,
            spiral_turns: 2.5,
        }
    }

    pub fn generate_looping_map(&mut self, complexity: u8) -> bool {
        self.nodes.clear();
        self.corridors.clear();
        self.loop_points.clear();
        self.discovery_counter = 0;

        let start_node = self.create_node("start", self.width / 2.0, self.height / 2.0);
        let start_id = start_node.node_id.clone();

        self.generate_main_loop(&start_id, complexity as i32);
        self.add_branches_and_dead_ends();
        self.create_loop_connections();

        if complexity >= 7 {
            self.generate_spiral_section();
        }

        true
    }

    fn create_node(&mut self, node_id: &str, x: f32, y: f32) -> MapNode {
        let node = MapNode::new(node_id, x, y);
        self.nodes.insert(node_id.to_string(), node.clone());
        node
    }

    fn generate_main_loop(&mut self, start_id: &str, segments: i32) {
        let mut current_id = start_id.to_string();
        let angle_step = (2.0 * std::f32::consts::PI) / segments as f32;
        let radius = 200.0f32;
        let mut rng = rand::thread_rng();

        for i in 0..segments {
            let angle = i as f32 * angle_step;
            let next_x = self.width / 2.0 + radius * angle.cos();
            let next_y = self.height / 2.0 + radius * angle.sin();

            let next_x = next_x.max(50.0).min(self.width - 50.0);
            let next_y = next_y.max(50.0).min(self.height - 50.0);

            let node_id = format!("loop_{}", i);
            self.create_node(&node_id, next_x, next_y);

            let corridor_type = if rng.gen::<f32>() < 0.6 {
                CorridorType::Curved
            } else {
                CorridorType::Straight
            };

            let current = self.nodes.get(&current_id).unwrap().clone();
            let next_node = self.nodes.get(&node_id).unwrap().clone();
            let dist = calculate_distance(&current, &next_node);

            let curvature = if corridor_type == CorridorType::Curved {
                rng.gen_range(0.3..0.8)
            } else {
                0.0
            };

            let segment = CorridorSegment {
                start_node: current_id.clone(),
                end_node: node_id.clone(),
                segment_type: corridor_type,
                length: dist,
                curvature,
            };

            self.corridors.push(segment);

            // 双向连接
            self.nodes.get_mut(&current_id).unwrap().connections.push(node_id.clone());
            self.nodes.get_mut(&node_id).unwrap().connections.push(current_id.clone());

            current_id = node_id;
            self.loop_points.push((next_x, next_y));
        }

        // 连接最后一个节点回到起点形成循环
        let last = self.nodes.get(&current_id).unwrap().clone();
        let start = self.nodes.get(start_id).unwrap().clone();
        let dist = calculate_distance(&last, &start);

        let final_segment = CorridorSegment {
            start_node: current_id.clone(),
            end_node: start_id.to_string(),
            segment_type: CorridorType::Curved,
            length: dist,
            curvature: 0.7,
        };

        self.corridors.push(final_segment);
        self.nodes.get_mut(&current_id).unwrap().connections.push(start_id.to_string());
        self.nodes.get_mut(start_id).unwrap().connections.push(current_id);
    }

    fn add_branches_and_dead_ends(&mut self) {
        let existing_ids: Vec<String> = self.nodes.keys().cloned().collect();
        let mut rng = rand::thread_rng();

        for node_id in existing_ids {
            if rng.gen::<f32>() >= self.branch_probability {
                continue;
            }

            let node = self.nodes.get(&node_id).unwrap().clone();

            let branch_angle = rng.gen_range(0.0..2.0 * std::f32::consts::PI);
            let branch_length = rng.gen_range(self.min_segment_length..self.max_segment_length);

            let branch_x = node.x + branch_length * branch_angle.cos();
            let branch_y = node.y + branch_length * branch_angle.sin();

            let branch_x = branch_x.max(30.0).min(self.width - 30.0);
            let branch_y = branch_y.max(30.0).min(self.height - 30.0);

            // 70% 概率连接回主路径
            let target_id: Option<String> = if rng.gen::<f32>() < 0.7 {
                let closest = self.find_closest_node(branch_x, branch_y, Some(&node_id));
                if let Some(closest) = closest {
                    let d = calculate_distance_coords(branch_x, branch_y, closest.x, closest.y);
                    if d < 200.0 {
                        Some(closest.node_id)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None  // 死胡同
            };

            let branch_node_id = format!("branch_{}", self.nodes.len());
            self.create_node(&branch_node_id, branch_x, branch_y);

            let segment_type = if rng.gen::<bool>() {
                CorridorType::Straight
            } else {
                CorridorType::Curved
            };

            let branch_node = self.nodes.get(&branch_node_id).unwrap().clone();
            let dist = calculate_distance(&node, &branch_node);
            let curvature = if segment_type == CorridorType::Curved {
                rng.gen_range(0.2..0.6)
            } else {
                0.0
            };

            let branch_segment = CorridorSegment {
                start_node: node_id.clone(),
                end_node: branch_node_id.clone(),
                segment_type,
                length: dist,
                curvature,
            };

            self.corridors.push(branch_segment);
            self.nodes.get_mut(&node_id).unwrap().connections.push(branch_node_id.clone());
            self.nodes.get_mut(&branch_node_id).unwrap().connections.push(node_id.clone());

            // 如果不是死胡同，建立回连接
            if let Some(target) = target_id {
                if target != node_id {
                    let target_node = self.nodes.get(&target).unwrap().clone();
                    let return_dist = calculate_distance(&branch_node, &target_node);

                    let return_segment = CorridorSegment {
                        start_node: branch_node_id.clone(),
                        end_node: target.clone(),
                        segment_type: CorridorType::Straight,
                        length: return_dist,
                        curvature: 0.0,
                    };

                    self.corridors.push(return_segment);
                    self.nodes.get_mut(&branch_node_id).unwrap().connections.push(target.clone());
                    self.nodes.get_mut(&target).unwrap().connections.push(branch_node_id.clone());
                }
            }
        }
    }

    fn create_loop_connections(&mut self) {
        let node_list: Vec<MapNode> = self.nodes.values().cloned().collect();
        let mut rng = rand::thread_rng();

        for i in 0..node_list.len().saturating_sub(2) {
            if rng.gen::<f32>() >= self.loop_probability {
                continue;
            }

            let node = &node_list[i];
            let mut candidates = Vec::new();

            for j in (i + 2)..node_list.len() {
                let candidate = &node_list[j];
                let dist = calculate_distance(node, candidate);
                if dist >= self.min_segment_length && dist <= self.max_segment_length * 1.5 {
                    candidates.push(candidate.node_id.clone());
                }
            }

            if !candidates.is_empty() {
                let selected = &candidates[rng.gen_range(0..candidates.len())];
                let selected_node = self.nodes.get(selected).unwrap().clone();
                let dist = calculate_distance(node, &selected_node);

                let loop_segment = CorridorSegment {
                    start_node: node.node_id.clone(),
                    end_node: selected.clone(),
                    segment_type: CorridorType::Curved,
                    length: dist,
                    curvature: rng.gen_range(0.4..0.9),
                };

                self.corridors.push(loop_segment);
                self.nodes.get_mut(&node.node_id).unwrap().connections.push(selected.clone());
                self.nodes.get_mut(selected).unwrap().connections.push(node.node_id.clone());
            }
        }
    }

    fn generate_spiral_section(&mut self) {
        let center_x = self.width / 2.0;
        let center_y = self.height / 2.0;
        let spiral_radius = 150.0f32;
        let spiral_segments = 8;

        let spiral_start = self.find_closest_node(center_x, center_y, None);
        let Some(start_node) = spiral_start else { return };
        let mut current_id = start_node.node_id;

        let angle_step = (2.0 * std::f32::consts::PI) / spiral_segments as f32;

        for i in 0..spiral_segments {
            let angle = i as f32 * angle_step;
            let current_radius = spiral_radius * (1.0 - i as f32 / spiral_segments as f32 * 0.7);

            let spiral_x = center_x + current_radius * (angle + std::f32::consts::PI).cos();
            let spiral_y = center_y + current_radius * (angle + std::f32::consts::PI).sin();

            let spiral_id = format!("spiral_{}", i);
            self.create_node(&spiral_id, spiral_x, spiral_y);

            let current = self.nodes.get(&current_id).unwrap().clone();
            let spiral_node = self.nodes.get(&spiral_id).unwrap().clone();
            let dist = calculate_distance(&current, &spiral_node);

            let spiral_segment = CorridorSegment {
                start_node: current_id.clone(),
                end_node: spiral_id.clone(),
                segment_type: CorridorType::Curved,
                length: dist,
                curvature: 0.8,
            };

            self.corridors.push(spiral_segment);
            self.nodes.get_mut(&current_id).unwrap().connections.push(spiral_id.clone());
            self.nodes.get_mut(&spiral_id).unwrap().connections.push(current_id.clone());

            current_id = spiral_id;
        }
    }

    fn find_closest_node(&self, x: f32, y: f32, exclude: Option<&str>) -> Option<MapNode> {
        let mut closest: Option<MapNode> = None;
        let mut min_dist = f32::MAX;

        for (node_id, node) in &self.nodes {
            if let Some(ex) = exclude {
                if node_id == ex {
                    continue;
                }
            }

            let d = calculate_distance_coords(x, y, node.x, node.y);
            if d < min_dist {
                min_dist = d;
                closest = Some(node.clone());
            }
        }

        closest
    }
}

fn calculate_distance(node1: &MapNode, node2: &MapNode) -> f32 {
    calculate_distance_coords(node1.x, node1.y, node2.x, node2.y)
}

fn calculate_distance_coords(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x1 - x2;
    let dy = y1 - y2;
    (dx * dx + dy * dy).sqrt()
}
