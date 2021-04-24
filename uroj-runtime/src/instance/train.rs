use std::time::Duration;

use tokio::time::sleep;

use super::{Instance, fsm::{Direction, GameFrame, MoveTrain, NodeID, NodeStatus}};

type TrainID = usize;

pub(crate) struct Train {
    id: TrainID,
    past_node: Vec<NodeID>,
}

impl Train {
    pub(crate) fn new(spawn_at: NodeID, id_fn: fn() -> TrainID) -> Self {
        Train {
            id: id_fn(),
            past_node: vec![spawn_at],
        }
    }

    pub(crate) fn curr_node(&self) -> NodeID {
        self.past_node.last().unwrap().clone()
    }

    //when node state is changed, call me
    pub(crate) async fn can_move_to(&self, target: &NodeID, ins: &Instance) -> bool {
        let arc_fsm = ins.fsm.clone();
        let fsm = arc_fsm.lock().await;
        let graph = &ins.graph;
        let curr = &self.curr_node();
        //鄰接保證物理上車可以移動
        //行车方向
        let dir = graph.direction(curr, target);
        if let None = dir {
            return false;
        }
        //若沒有防護信號機則無約束，若有則檢查點亮的信號是否允許進入

        let target_node = fsm.node(*target);
        let pro_sgn_id = match dir.unwrap() {
            Direction::Left => target_node.right_sgn_id.as_ref(),
            Direction::Right => target_node.left_sgn_id.as_ref(),
        };

        pro_sgn_id.map_or(true, |s| fsm.sgn(&s).is_allowed())
    }

    async fn move_to(&mut self, target: &NodeID, ins: &Instance) {
        let arc_fsm = ins.fsm.clone();
        let mut fsm = arc_fsm.lock().await;
        let from = &self.curr_node();

        //入口防護信號燈
        fsm.node_mut(*target).state = NodeStatus::Occupied; //下一段占用
        fsm.node_mut(*from).state = NodeStatus::Vacant; // 上一段出清
        fsm.node_mut(*from).once_occ = true; // 上一段曾占用
        self.past_node.push(target.clone());

        //三點檢查
        // if  {}
        sleep(Duration::from_secs(3)).await;
    }

    //when node state is changed, call me
    pub(crate) async fn try_move_to(&self, target: &NodeID, ins: &Instance) -> Option<GameFrame> {
        if self.can_move_to(target, ins).await {
            return Some(GameFrame::MoveTrain(MoveTrain {
                id: self.id,
                node_id: *target,
                process: 0.,
            }));
        }
        None
    }
}