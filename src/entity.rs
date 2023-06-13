
#[derive(Debug, Clone)]
pub struct Entity {
    pub color: String,
    pub position: (i32, i32)
}

impl Entity {
    pub fn to_json(&self) -> String {
        format!("{{\"position\": {{ \"x\": {}, \"y\": {} }}, \"color\": \"{}\"}}", 
                self.position.0, self.position.1,
                self.color
        )
    }
}
