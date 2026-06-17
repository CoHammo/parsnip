use std::collections::HashMap;

pub struct Tags {
    count: u32,
    names: HashMap<Tag, String>,
    ids: HashMap<String, Tag>,
}
impl Tags {
    pub fn new() -> Self {
        let mut names = HashMap::new();
        names.insert(Tag(0), "Text".into());
        let mut ids = HashMap::new();
        ids.insert("Text".into(), Tag(0));
        Self {
            count: 1,
            names,
            ids,
        }
    }

    pub fn none(&self) -> Tag {
        Tag(0)
    }

    pub fn add(&mut self, name: &str) {
        self.names.insert(Tag(self.count), name.into());
        self.ids.insert(name.into(), Tag(self.count));
        self.count += 1;
    }

    pub fn tag(&mut self, name: &str) -> Tag {
        if let Some(tag) = self.ids.get(name) {
            *tag
        } else {
            self.add(name);
            self.tag(name)
        }
    }

    pub fn get(&self, tag: Tag) -> &str {
        if let Some(name) = self.names.get(&tag) {
            name
        } else {
            ""
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Tag(u32);
