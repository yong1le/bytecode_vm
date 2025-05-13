#[derive(Debug, Clone, Copy)]
pub enum VMUpvalue {
    Open(usize),   // Index into stack
    Closed(usize), // Index into heap
}

impl VMUpvalue {}
