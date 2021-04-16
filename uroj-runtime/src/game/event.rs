use super::components::NodeID;

trait Event {
    fn handle(&self);
}

pub struct TrainMoveEvent{
    from: NodeID, 
    to: NodeID,
}

impl Event for TrainMoveEvent {
    fn handle(&self) {
        
    }
}