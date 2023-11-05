pub mod client;
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub enum Data {
    Text(String),
    File { filename: String, file: Vec<u8> },
}
impl ToString for Data {
    fn to_string(&self) -> String {
        match self {
            Data::Text(s) => s.clone(),
            Data::File { filename, file } => format!("{}: {}", filename, file.len()),
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Message {
    pub id: u32,
    pub data: Data,
}
impl Message {
    pub fn new(id: u32, data: Data) -> Self {
        Self { id, data }
    }
}
impl ToString for Message {
    fn to_string(&self) -> String {
        return format!("{}: {}", self.id, self.data.to_string());
    }
}
