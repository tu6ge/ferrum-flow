use futures::channel::mpsc::UnboundedSender;

use crate::{GraphChange, GraphOp};

pub trait SyncPlugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, change_sender: UnboundedSender<GraphChange>);

    fn process_intent(&self, op: GraphOp);

    fn undo(&mut self);
    fn redo(&mut self);

    fn get_full_snapshot(&self) -> Vec<GraphChange>;
}
