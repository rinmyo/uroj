use std::{collections::HashMap, time::Duration};

use super::graph::StationGraph;
use crate::{kafka::send_message, raw_station::*};
use async_graphql::*;
use rdkafka::producer::FutureProducer;
use serde::{Deserialize, Serialize};
use strum_macros::*;
use tokio::time::sleep;

//實例狀態機
pub(crate) struct InstanceFSM {
    pub(crate) sgns: HashMap<String, Signal>,
    pub(crate) nodes: HashMap<NodeID, Node>,
}

impl InstanceFSM {
    pub(crate) fn new(
        sgn_vec: &Vec<RawSignal>,
        node_vec: &Vec<RawNode>,
        graph: &StationGraph,
    ) -> Result<InstanceFSM, String> {
        let mut sgns: HashMap<String, Signal> =
            sgn_vec.iter().map(|s| (s.id.clone(), s.into())).collect();
        let mut nodes: HashMap<NodeID, Node> = node_vec.iter().map(|n| (n.id, n.into())).collect();

        for s in sgn_vec {
            let pid = s.protect_node_id;
            let tid = s.toward_node_id;

            let dir = graph
                .direction(&pid, &tid)
                .or_else(|| match s.is_left {
                    Some(true) => Some(Direction::Left),
                    Some(false) => Some(Direction::Right),
                    None => None,
                })
                .ok_or(format!("connot parse the direction of signal {}", s.id))?;

            let sgn = sgns.get_mut(&s.id).unwrap();
            (*sgn).direction = dir.clone();

            let p_node = nodes
                .get_mut(&pid)
                .ok_or(format!("unknown node id: {}", pid))?;

            match dir {
                Direction::Left => (*p_node).left_sgn_id = Some(s.id.clone()),
                Direction::Right => (*p_node).right_sgn_id = Some(s.id.clone()),
            }
        }

        Ok(InstanceFSM {
            sgns: sgns,
            nodes: nodes,
        })
    }

    pub(crate) fn node_mut(&mut self, id: NodeID) -> &mut Node {
        self.nodes.get_mut(&id).expect("cannot get node")
    }

    pub(crate) fn node(&self, id: NodeID) -> &Node {
        &self.nodes[&id]
    }

    pub(crate) fn sgn(&self, id: &str) -> &Signal {
        &self.sgns[id]
    }

    pub(crate) fn sgn_mut(&mut self, id: &str) -> &mut Signal {
        self.sgns.get_mut(id).expect("cannot get node")
    }
}

pub(crate) type NodeID = usize;
//Node 狀態機，沒有耦合信息
pub(crate) struct Node {
    pub(crate) node_id: NodeID,
    pub(crate) used_count: u32, //被征用计数，每次征用则INC，每次作为S扩展集中的点被解除征用则减1，为0则说明未被征用
    pub(crate) state: NodeStatus,
    pub(crate) kind: RawNodeKind,
    pub(crate) once_occ: bool,
    pub(crate) is_lock: bool,
    pub(crate) left_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
    pub(crate) right_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
}

impl Node {
    pub(crate) async fn unlock(&mut self, producer: &FutureProducer) {
        sleep(Duration::from_secs(3)).await;
        self.is_lock = false;
        self.sync_state(producer).await;
    }

    pub(crate) async fn lock(&mut self, producer: &FutureProducer) {
        self.is_lock = true;
        self.sync_state(producer).await;
    }

    async fn sync_state(&mut self, producer: &FutureProducer) {
        let state = if self.is_lock {
            NodeStatus::Lock
        } else {
            self.state
        };
        GameFrame::UpdateNode(UpdateNode {
            id: self.node_id,
            state: state,
        })
        .send_via(producer)
        .await;
    }
}

impl From<&RawNode> for Node {
    fn from(data: &RawNode) -> Self {
        Node {
            node_id: data.id,
            used_count: 0,
            state: Default::default(),
            once_occ: false,
            is_lock: false,
            left_sgn_id: None,
            right_sgn_id: None,
            kind: data.node_kind.clone(),
        }
    }
}

//事实上的动态状态
//由车辆的位置和预设变量决定，是轨道电路的表征
// 在毕业设计中使用状态转移图，可以凑字数
#[derive(Clone, Eq, PartialEq, Display, Serialize, Deserialize, Enum, Copy)]
pub(crate) enum NodeStatus {
    Occupied,   //占用，赤
    Unexpected, //异常，条
    Vacant,     //空闲，蓝
    Lock,
}

impl Default for NodeStatus {
    fn default() -> Self {
        NodeStatus::Vacant
    }
}

pub(crate) struct Signal {
    pub(crate) signal_id: String,
    pub(crate) filament_status: (FilamentStatus, FilamentStatus),
    pub(crate) state: SignalStatus,
    pub(crate) used_flag: bool,     //征用
    pub(crate) kind: RawSignalKind, //因为逻辑不需要变化
    pub(crate) toward_node_id: NodeID,
    pub(crate) protect_node_id: NodeID,
    pub(crate) direction: Direction, //朝向
}

impl From<&RawSignal> for Signal {
    fn from(data: &RawSignal) -> Self {
        let filament_status = match data.sgn_kind {
            //调车信号机只有一个灯丝
            RawSignalKind::ShuntingSignal => (Default::default(), FilamentStatus::None),
            _ => (Default::default(), Default::default()),
        };

        let state = match data.sgn_kind {
            RawSignalKind::ShuntingSignal => SignalStatus::A,
            _ => SignalStatus::H,
        };

        Signal {
            signal_id: data.id.clone(),
            filament_status: filament_status,
            state: state,
            used_flag: false,
            kind: data.sgn_kind,
            toward_node_id: data.toward_node_id,
            protect_node_id: data.protect_node_id,
            direction: Direction::Left,
        }
    }
}

impl Signal {
    pub(crate) fn is_allowed(&self) -> bool {
        match self.state {
            SignalStatus::L
            | SignalStatus::U
            | SignalStatus::B
            | SignalStatus::UU
            | SignalStatus::LU
            | SignalStatus::LL
            | SignalStatus::US
            | SignalStatus::HB => true,
            SignalStatus::A | SignalStatus::H | SignalStatus::OFF => false,
        }
    }

    pub(crate) async fn update(&mut self, state: SignalStatus, producer: &FutureProducer) {
        self.state = state;

        GameFrame::UpdateSignal(UpdateSignal {
            id: self.signal_id.clone(),
            state: state,
        })
        .send_via(producer)
        .await;
    }

    pub(crate) async fn protect(&mut self, producer: &FutureProducer) {
        let new_state = match self.kind {
            RawSignalKind::HomeSignal => SignalStatus::H,
            RawSignalKind::StartingSignal => SignalStatus::H,
            RawSignalKind::ShuntingSignal => SignalStatus::A,
        };
        self.update(new_state, producer).await;
    }

    //开放接车进路
    pub(crate) async fn open_recv(&mut self, goal_kind: RawNodeKind, producer: &FutureProducer) {
        let new_state = match goal_kind {
            RawNodeKind::Mainline => SignalStatus::U,
            RawNodeKind::Siding => SignalStatus::UU,
            RawNodeKind::Siding18 => SignalStatus::US,
            RawNodeKind::Normal => return,
        };
        self.update(new_state, producer).await;
    }

    //开放发车进路信号
    pub(crate) async fn open_send(&mut self, producer: &FutureProducer) {
        self.update(SignalStatus::L, producer).await;
    }

    //开放通過進路信號
    pub(crate) async fn open_pass(&mut self, producer: &FutureProducer) {
        self.update(SignalStatus::L, producer).await;
    }

    //开放调车进路信号
    pub(crate) async fn open_shnt<'a>(&mut self, producer: &FutureProducer) {
        self.update(SignalStatus::L, producer).await;
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum SignalStatus {
    L,
    U,
    H,
    B,
    A,
    UU,
    LU,
    LL,
    US,
    HB,
    OFF,
}

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum FilamentStatus {
    Normal,
    Fused,
    None,
}

impl Default for FilamentStatus {
    fn default() -> Self {
        FilamentStatus::Normal
    }
}

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Direction {
    Left,
    Right,
}

impl Direction {
    pub(crate) fn reverse(&self) -> Self {
        match self {
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(SimpleObject)]
pub(crate) struct GlobalStatus {
    pub(crate) nodes: Vec<UpdateNode>,
    pub(crate) signals: Vec<UpdateSignal>,
}

#[derive(Union, Clone, Serialize, Deserialize)]
pub(crate) enum GameFrame {
    UpdateSignal(UpdateSignal),
    UpdateNode(UpdateNode),
    MoveTrain(MoveTrain),
}

impl GameFrame {
    pub(crate) async fn send_via(&self, producer: &FutureProducer) {
        let msg = serde_json::to_string(self).expect("cannot parse frame");
        send_message(producer, msg).await;
    }
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct UpdateSignal {
    pub(crate) id: String,
    pub(crate) state: SignalStatus,
}

#[derive(SimpleObject, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct UpdateNode {
    pub(crate) id: NodeID,
    pub(crate) state: NodeStatus,
}
#[derive(SimpleObject, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct MoveTrain {
    pub(crate) id: usize,
    pub(crate) node_id: NodeID,
    pub(crate) process: f64,
}
