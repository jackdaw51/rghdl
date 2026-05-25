//Let's make this shi arena allocated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityId(pub u32);

#[derive(Debug, Clone)]
pub struct Entity<'a> {
    pub name: &'a str,
    pub ports_start: PortId,
    pub ports_end: PortId, 
}

#[derive(Debug, Clone)]
pub struct Port<'a> {
    pub name: &'a str,
    pub mode: PortMode,
    pub port_type: &'a str, 
}

#[derive(Debug, Clone, PartialEq)]
pub enum PortMode {
    In,
    Out,
    InOut,
    Buffer,
}

#[derive(Default, Debug)]
pub struct AstArena<'a> {
    pub entities: Vec<Entity<'a>>,
    pub ports: Vec<Port<'a>>,
}

impl<'a> AstArena<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_port(&mut self, port: Port<'a>) -> PortId {
        let id = self.ports.len() as u32;
        self.ports.push(port);
        PortId(id)
    }

    pub fn alloc_entity(&mut self, entity: Entity<'a>) -> EntityId {
        let id = self.entities.len() as u32;
        self.entities.push(entity);
        EntityId(id)
    }
}