use futures::channel::mpsc::UnboundedSender;
use gpui::AnyElement;

use crate::{GraphChange, GraphOp, RenderContext};

pub trait SyncPlugin {
    fn name(&self) -> &'static str;

    fn setup(&mut self, change_sender: UnboundedSender<GraphChange>);

    fn process_intent(&self, op: GraphOp);

    fn undo(&mut self);
    fn redo(&mut self);

    fn get_full_snapshot(&self) -> Vec<GraphChange>;

    fn render(&mut self, _ctx: &mut RenderContext) -> Vec<AnyElement> {
        vec![]
    }
}
