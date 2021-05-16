use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::raw_station::*;
use async_graphql::*;
use log::debug;
use serde::{Deserialize, Serialize};
use strum_macros::*;
use tokio::{
    sync::{broadcast::Sender, Mutex, MutexGuard},
    time::delay_for,
};

use super::{topo::Topo, FrameSender, GameFrame};

//實例狀態機
pub(crate) struct InstanceFSM {
    pub(crate) sgns: HashMap<String, Mutex<Signal>>,
    pub(crate) nodes: HashMap<NodeID, Mutex<Node>>,
    pub(crate) trains: Arc<Mutex<Vec<Arc<Mutex<Train>>>>>,
}

impl InstanceFSM {
    pub(crate) async fn node(&self, id: NodeID) -> MutexGuard<'_, Node> {
        debug!("req lock {}", id);
        self.nodes
            .get(&id)
            .expect(&format!("unknown node: {}", id))
            .lock()
            .await
    }

    pub(crate) async fn sgn(&self, id: &str) -> MutexGuard<'_, Signal> {
        debug!("find sgn: {}", id);
        self.sgns
            .get(id)
            .expect(&format!("unknown signal: {}", id))
            .lock()
            .await
    }

    pub(crate) async fn spawn_train(
        &mut self,
        node: NodeID,
        sender: &FrameSender,
    ) -> Arc<Mutex<Train>> {
        let mut trains = self.trains.lock().await;
        let id = trains.len() + 1;
        let new_train = Train::new(node, id, sender).await;
        let arc_train = Arc::new(new_train);
        let cloned_train = arc_train.clone();
        trains.push(arc_train);
        cloned_train
    }

    pub(crate) async fn get_global_status(&self) -> GlobalStatus {
        let mut signals = vec![];
        for s in self.sgns.values() {
            let s = s.lock().await;
            signals.push(s.to_update_signal())
        }
        let mut nodes = vec![];
        for n in self.nodes.values() {
            let n = n.lock().await;
            nodes.push(n.to_update_node())
        }
        GlobalStatus {
            nodes: nodes,
            signals: signals,
        }
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
    pub(crate) len: f64,                     //全长
    pub(crate) left_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
    pub(crate) right_sgn_id: Option<String>, //两端的防护信号机，只有防护自己的信号机才在这里//點燈時用的
}

impl Node {
    pub(crate) async fn unlock(&mut self, sender: &FrameSender) {
        self.is_lock = false;
        self.sync_state(sender).await;
    }

    pub(crate) async fn lock(&mut self, sender: &FrameSender) {
        self.is_lock = true;
        self.sync_state(sender).await;
    }

    async fn sync_state(&mut self, sender: &FrameSender) {
        GameFrame::UpdateNode(self.to_update_node())
            .send_via(sender)
            .await;
    }

    fn to_update_node(&self) -> UpdateNode {
        UpdateNode {
            id: self.node_id.clone(),
            state: if self.is_lock {
                NodeStatus::Lock
            } else {
                self.state
            },
        }
    }
}

impl From<&RawNode> for Node {
    fn from(data: &RawNode) -> Self {
        let (x1, y1) = data.line.0;
        let (x2, y2) = data.line.1;

        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        Node {
            node_id: data.id,
            used_count: 0,
            state: Default::default(),
            once_occ: false,
            is_lock: false,
            len: len,
            left_sgn_id: None,  //先缺省，之後推斷
            right_sgn_id: None, //先缺省之後推斷
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
    pub(crate) id: String,
    pub(crate) filament_status: (FilamentStatus, FilamentStatus),
    pub(crate) state: SignalStatus,
    pub(crate) kind: RawSignalKind, //因为逻辑不需要变化
    pub(crate) protect_node_id: NodeID,
    pub(crate) toward_node_id: NodeID,
    pub(crate) dir: Direction, //朝向
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
            id: data.id.clone(),
            filament_status: filament_status,
            state: state,
            kind: data.sgn_kind,
            protect_node_id: data.protect_node_id,
            toward_node_id: data.toward_node_id,
            dir: Direction::Left, //缺省
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

    pub(crate) fn to_update_signal(&self) -> UpdateSignal {
        UpdateSignal {
            id: self.id.clone(),
            state: self.state,
        }
    }

    pub(crate) async fn update(&mut self, state: SignalStatus, sender: &FrameSender) {
        self.state = state;

        GameFrame::UpdateSignal(self.to_update_signal())
            .send_via(sender)
            .await;
    }

    pub(crate) async fn protect(&mut self, sender: &FrameSender) {
        let new_state = match self.kind {
            RawSignalKind::HomeSignal => SignalStatus::H,
            RawSignalKind::StartingSignal => SignalStatus::H,
            RawSignalKind::ShuntingSignal => SignalStatus::A,
        };
        self.update(new_state, sender).await;
    }

    //开放接车进路
    pub(crate) async fn open_recv(&mut self, goal_kind: RawNodeKind, sender: &FrameSender) {
        let new_state = match goal_kind {
            RawNodeKind::Mainline => SignalStatus::U,
            RawNodeKind::Siding => SignalStatus::UU,
            RawNodeKind::Siding18 => SignalStatus::US,
            RawNodeKind::Normal => return,
        };
        self.update(new_state, sender).await;
    }

    //完全开放信号
    pub(crate) async fn open(&mut self, sender: &FrameSender) {
        let new_state = match self.kind {
            RawSignalKind::HomeSignal => SignalStatus::L,
            RawSignalKind::StartingSignal => SignalStatus::L,
            RawSignalKind::ShuntingSignal => SignalStatus::B,
        };
        self.update(new_state, sender).await;
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

#[derive(Clone, SimpleObject, Serialize, Deserialize)]
pub(crate) struct GlobalStatus {
    pub(crate) nodes: Vec<UpdateNode>,
    pub(crate) signals: Vec<UpdateSignal>,
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
    pub(crate) dir: RawDirection,
}

type TrainID = usize;

pub(crate) struct Train {
    pub(crate) id: TrainID,
    process: f64,
    dir: RawDirection,
    pub(crate) past_node: Vec<NodeID>,
}

impl Train {
    pub(crate) async fn new(spawn_at: NodeID, id: TrainID, sender: &FrameSender) -> Mutex<Self> {
        let train = Train {
            id: id,
            process: 0.5,
            past_node: vec![spawn_at],
            dir: RawDirection::Left,
        };

        train.send_states(sender).await;
        let arc_train = Mutex::new(train);
        arc_train
    }

    pub(crate) fn turn_direction(&mut self, dir: RawDirection) {
        self.dir = dir;
        self.process = 1. - self.process;
    }

    pub(crate) async fn send_states(&self, sender: &FrameSender) {
        GameFrame::MoveTrain(MoveTrain {
            id: self.id,
            node_id: self.curr_node(),
            process: self.process,
            dir: self.dir,
        })
        .send_via(sender)
        .await;
    }

    pub(crate) fn curr_node(&self) -> NodeID {
        self.past_node.last().unwrap().clone()
    }

    //when node state is changed, call me
    pub(crate) async fn can_move_to(&self, target: NodeID, topo: &Topo, fsm: &InstanceFSM) -> bool {
        let curr = self.curr_node();
        //鄰接保證物理上車可以移動
        //行车方向, 这是边，没有则不邻接，物理上不可移动
        let dir = topo.direction(curr, target);
        if let None = dir {
            return false;
        }

        //若沒有防護信號機則無約束，若有則檢查點亮的信號是否允許進入
        let target_node = fsm.node(target).await;
        let pro_sgn_id = match dir.unwrap() {
            RawDirection::Left => target_node.right_sgn_id.as_ref(),
            RawDirection::Right => target_node.left_sgn_id.as_ref(),
        };

        if let Some(s) = pro_sgn_id {
            if !fsm.sgn(s).await.is_allowed() {
                return false;
            }
        }

        true
    }

    async fn move_to(&mut self, target: NodeID, fsm: &InstanceFSM) {
        let from = self.curr_node();
        debug!("train move to {}", target);

        //入口防護信號燈
        fsm.node(target).await.state = NodeStatus::Occupied; //下一段占用
        let mut from = fsm.node(from).await;
        from.state = NodeStatus::Vacant; // 上一段出清
        from.once_occ = true; // 上一段曾占用
        self.past_node.push(target.clone());
        self.process = 0.;

        //三點檢查
        // if  {}
        delay_for(Duration::from_secs(3)).await;
    }

    //when node state is changed, call me
    pub(crate) async fn try_next_step(
        &mut self,
        target: NodeID,
        fsm: &InstanceFSM,
        topo: &Topo,
        sender: &Sender<GameFrame>,
    ) {
        if self.process < 1. {
            let tgt_node = fsm.node(target).await;
            self.process += 1. / tgt_node.len;
        } else if self.can_move_to(target, topo, fsm).await {
            debug!("test move to {}", target);
            self.move_to(target, fsm).await;
        } else {
            return;
        }

        self.send_states(sender).await;
    }
}
