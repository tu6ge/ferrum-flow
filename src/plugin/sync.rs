use tokio::sync::mpsc::Sender;

use crate::{GraphChange, GraphOp};

pub trait SyncPlugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, change_sender: Sender<GraphChange>);

    fn process_intent(&self, op: GraphOp);

    fn undo(&mut self);
    fn redo(&mut self);

    fn get_full_snapshot(&self) -> Vec<GraphChange>;
}
