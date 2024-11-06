pub struct Quest {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

impl Quest {
    pub fn new(name: &str, description: &str) -> Quest {
        Quest {
            name: name.to_string(),
            description: description.to_string(),
            completed: false,
        }
    }

    pub fn complete(&mut self) {
        self.completed = true;
        println!("Quest completed: {}", self.name);
    }
}
