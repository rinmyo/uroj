use std::{
    collections::{HashMap, HashSet},
    time::Duration,
};
use strum_macros::*;

use async_graphql::Enum;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, Sender},
    time::sleep,
};
use uroj_common::rpc::{Node as StaticNode, NodeKind, Signal as StaticSignal, SignalKind};

use crate::models::{GameFrame, UpdateNode, UpdateSignal};

use super::{
    event::TrainMoveEvent,
    instance::{Direction, Instance},
};

pub(crate) type NodeID = usize;
//Node 狀態機，沒有耦合信息
pub(crate) struct Node {
    pub(crate) node_id: NodeID,
    pub(crate) used_count: u32, //被征用计数，每次征用则INC，每次作为S扩展集中的点被解除征用则减1，为0则说明未被征用
    pub(crate) state: NodeStatus,
    pub(crate) kind: NodeKind,
    pub(crate) is_lock: bool,
    pub(crate) once_occ: bool,
    pub(crate) left_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
    pub(crate) right_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
}

impl Node {
    pub(crate) async fn unlock(&mut self, sender: &Sender<GameFrame>) {
        sleep(Duration::from_secs(3)).await;
        self.is_lock = false;
        sender.send(GameFrame::UpdateNode(UpdateNode {
            id: self.node_id,
            state: crate::models::NodeStatus::Vacant,
        }));
    }

    pub(crate) fn lock(&mut self, sender: &Sender<GameFrame>) {
        self.is_lock = true;
        sender.send(GameFrame::UpdateNode(UpdateNode {
            id: self.node_id,
            state: crate::models::NodeStatus::Lock,
        }));
    }
}

impl From<&StaticNode> for Node {
    fn from(data: &StaticNode) -> Self {
        Node {
            node_id: data.node_id,
            used_count: 0,
            state: Default::default(),
            is_lock: false,
            once_occ: false,
            left_sgn_id: None,
            right_sgn_id: None,
            kind: data.node_kind.clone(),
        }
    }
}

//由车辆的位置和预设变量决定，是轨道电路的表征
// 在毕业设计中使用状态转移图，可以凑字数
#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize, Enum, Display, EnumString)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub(crate) enum NodeStatus {
    Occupied,   //占用，赤
    Unexpected, //异常，条
    Vacant,     //空闲，蓝
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
    pub(crate) used_flag: bool, //征用
    pub(crate) kind: SignalKind,
    pub(crate) toward_node_id: NodeID,
    pub(crate) protect_node_id: NodeID,
    pub(crate) direction: Direction, //朝向
}

impl From<&StaticSignal> for Signal {
    fn from(data: &StaticSignal) -> Self {
        let filament_status = match data.sgn_type {
            //调车信号机只有一个灯丝
            SignalKind::ShuntingSignal => (Default::default(), FilamentStatus::None),
            _ => (Default::default(), Default::default()),
        };

        let state = match data.sgn_type {
            SignalKind::ShuntingSignal => SignalStatus::A,
            _ => SignalStatus::H,
        };

        Signal {
            signal_id: data.id.clone(),
            filament_status: filament_status,
            state: state,
            used_flag: false,
            kind: data.sgn_type.clone(),
            toward_node_id: data.toward_node_id,
            protect_node_id: data.protect_node_id,
            direction: Direction::Left,
        }
    }
}

impl Signal {
    fn is_allowed(&self) -> bool {
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

    fn update(&mut self, state: SignalStatus, sender: &Sender<GameFrame>) {
        self.state = state;

        sender.send(GameFrame::UpdateSignal(UpdateSignal {
            id: self.signal_id.clone(),
            state: state,
        }));
    }

    pub(crate) fn protect(&mut self, sender: &Sender<GameFrame>) {
        let new_state = match self.kind {
            SignalKind::HomeSignal => SignalStatus::H,
            SignalKind::StartingSignal => SignalStatus::H,
            SignalKind::ShuntingSignal => SignalStatus::A,
        };
        self.update(new_state, sender);
    }

    //开放接车进路
    pub(crate) fn open_recv(&mut self, goal_kind: NodeKind, sender: &Sender<GameFrame>) {
        let new_state = match goal_kind {
            NodeKind::Mainline => SignalStatus::U,
            NodeKind::Siding => SignalStatus::UU,
            NodeKind::Siding18 => SignalStatus::US,
            NodeKind::Normal => return,
        };
        self.update(new_state, sender);
    }

    //开放发车进路信号
    pub(crate) fn open_send(&mut self, sender: &Sender<GameFrame>) {
        self.update(SignalStatus::L, sender);
    }

    //开放通過進路信號
    pub(crate) fn open_pass(&mut self, sender: &Sender<GameFrame>) {
        self.update(SignalStatus::L, sender);
    }

    //开放调车进路信号
    pub(crate) fn open_shnt<'a>(
        sgns: impl Iterator<Item = &'a mut Self>,
        sender: &Sender<GameFrame>,
    ) {
        sgns.for_each(|sgn| sgn.update(SignalStatus::L, sender));
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

pub(crate) struct Train {
    past_node: Vec<NodeID>,
}

impl Train {
    pub(crate) fn new(spawn_at: NodeID) -> Self {
        Train {
            past_node: vec![spawn_at],
        }
    }

    pub(crate) fn curr_node(&self) -> NodeID {
        self.past_node.last().unwrap().clone()
    }

    //when node state is changed, call me
    pub(crate) fn can_move_to(&self, target: &NodeID, ins: &Instance) -> bool {
        let fsm = &ins.fsm;
        let graph = &ins.graph;
        let curr = &self.curr_node();
        //鄰接保證物理上車可以移動
        //行车方向
        let dir = graph.direction(curr, target);
        if let None = dir {
            return false;
        }
        //若沒有防護信號機則無約束，若有則檢查點亮的信號是否允許進入

        let target_node = fsm.node(target);
        let pro_sgn_id = match dir.unwrap() {
            Direction::Left => target_node.right_sgn_id.as_ref(),
            Direction::Right => target_node.left_sgn_id.as_ref(),
        };

        pro_sgn_id.map_or(true, |s| fsm.sgn(&s).is_allowed())
    }

    async fn move_to(&mut self, target: &NodeID, ins: &Instance) {
        let fsm = &ins.fsm;
        let from = &self.curr_node();

        //入口防護信號燈
        fsm.node_mut(target).state = NodeStatus::Occupied; //下一段占用
        fsm.node_mut(from).state = NodeStatus::Vacant; // 上一段出清
        fsm.node_mut(from).once_occ = true; // 上一段曾占用
        self.past_node.push(target.clone());

        //三點檢查
        // if  {}
        sleep(Duration::from_secs(3)).await;
    }

    //when node state is changed, call me
    pub(crate) fn try_move_to(&self, target: &NodeID, ins: &Instance) -> Option<TrainMoveEvent> {
        if self.can_move_to(target, ins) {
            return Some(TrainMoveEvent {
                from: self.curr_node(),
                to: target.clone(),
            });
        }
        None
    }
}
