use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::raw_station::*;
use async_graphql::*;
use serde::{Deserialize, Serialize};
use strum_macros::*;
use tokio::{
    sync::{broadcast::Sender, Mutex, MutexGuard},
    time::sleep,
};

use super::{topo::Topo, FrameSender, GameFrame, Instance};

//實例狀態機
pub(crate) struct InstanceFSM {
    pub(crate) sgns: HashMap<String, Mutex<Signal>>,
    pub(crate) nodes: HashMap<NodeID, Mutex<Node>>,
    pub(crate) trains: Arc<Mutex<Vec<Arc<Mutex<Train>>>>>,
}

impl InstanceFSM {
    pub(crate) async fn node(&self, id: NodeID) -> MutexGuard<'_, Node> {
        self.nodes.get(&id).expect("cannot get node").lock().await
    }

    pub(crate) async fn sgn(&self, id: &str) -> MutexGuard<'_, Signal> {
        self.sgns.get(id).expect("cannot get node").lock().await
    }

    pub(crate) async fn spawn_train(&mut self, node: NodeID) -> Arc<Mutex<Train>> {
        let mut trains = self.trains.lock().await;
        let id = trains.len() + 1;
        let new_train = Arc::new(Train::new(node, id));
        let cloned_train = new_train.clone();
        trains.push(new_train);
        cloned_train
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
        let state = if self.is_lock {
            NodeStatus::Lock
        } else {
            self.state
        };
        GameFrame::UpdateNode(UpdateNode {
            id: self.node_id,
            state: state,
        })
        .send_via(sender)
        .await;
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

    pub(crate) async fn update(&mut self, state: SignalStatus, sender: &FrameSender) {
        self.state = state;

        GameFrame::UpdateSignal(UpdateSignal {
            id: self.id.clone(),
            state: state,
        })
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

    //开放发车进路信号
    pub(crate) async fn open_send(&mut self, sender: &FrameSender) {
        self.update(SignalStatus::L, sender).await;
    }

    //开放通過進路信號
    pub(crate) async fn open_pass(&mut self, sender: &FrameSender) {
        self.update(SignalStatus::L, sender).await;
    }

    //开放调车进路信号
    pub(crate) async fn open_shnt<'a>(&mut self, sender: &FrameSender) {
        self.update(SignalStatus::L, sender).await;
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
    past_node: Vec<NodeID>,
}

impl Train {
    pub(crate) fn new(spawn_at: NodeID, id: TrainID) -> Mutex<Self> {
        let train = Train {
            id: id,
            process: 0.5,
            past_node: vec![spawn_at],
        };

        let arc_train = Mutex::new(train);
        arc_train
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

        //入口防護信號燈
        fsm.node(target).await.state = NodeStatus::Occupied; //下一段占用
        let mut from = fsm.node(from).await;
        from.state = NodeStatus::Vacant; // 上一段出清
        from.once_occ = true; // 上一段曾占用
        self.past_node.push(target.clone());
        self.process = 0.;

        //三點檢查
        // if  {}
        sleep(Duration::from_secs(3)).await;
    }

    //when node state is changed, call me
    pub(crate) async fn try_next_step(
        &mut self,
        target: NodeID,
        dir: RawDirection,
        fsm: &InstanceFSM,
        topo: &Topo,
        sender: &Sender<GameFrame>,
    ) {
        if self.process < 1. {
            self.process += 0.001;
        } else if self.can_move_to(target, topo, fsm).await {
            self.move_to(target, fsm).await;
        } else {
            return;
        }

        GameFrame::MoveTrain(MoveTrain {
            id: self.id,
            node_id: target,
            process: self.process,
            dir: dir,
        })
        .send_via(sender)
        .await;
    }
}
