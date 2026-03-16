use crate::{Graph, Viewport};

pub trait Command {
    fn name(&self) -> &'static str;
    fn execute(&mut self, ctx: &mut CanvasState);
    fn undo(&mut self, ctx: &mut CanvasState);
}

pub struct CanvasState<'a> {
    pub graph: &'a mut Graph,
    pub viewport: &'a mut Viewport,
}

const MAX_HISTORY: usize = 100;

pub struct History {
    undo_stack: Vec<Box<dyn Command>>,
    redo_stack: Vec<Box<dyn Command>>,
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, state: &mut CanvasState) {
        command.execute(state);

        self.undo_stack.push(command);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, state: &mut CanvasState) {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(state);
            self.redo_stack.push(cmd);
        }
    }

    pub fn redo(&mut self, state: &mut CanvasState) {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(state);
            self.undo_stack.push(cmd);
        }
    }
}

pub struct CompositeCommand {
    commands: Vec<Box<dyn Command>>,
}

impl Command for CompositeCommand {
    fn name(&self) -> &'static str {
        "composite"
    }
    fn execute(&mut self, state: &mut CanvasState) {
        for cmd in &mut self.commands {
            cmd.execute(state);
        }
    }

    fn undo(&mut self, state: &mut CanvasState) {
        for cmd in self.commands.iter_mut().rev() {
            cmd.undo(state);
        }
    }
}
