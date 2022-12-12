
pub struct Logger {
    pub content: String,
}

impl Logger {
    pub fn new(content: String) -> Self {
        Self { content }
    }

    pub fn log(&self) {
        let now = chrono::Local::now();
        let file_name = now.format("%Y-%m-%d_%H-%M-%S").to_string();
        
        let path = format!("storage/logs/{}.log", file_name);

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        let content = format!("{} {}", self.content, "\n");

        std::io::Write::write_all(&mut file, content.as_bytes()).unwrap();
    }
}

pub struct LoggerLayer {
    pub content: String,
}

impl LoggerLayer {
    pub fn new(content: String) -> Self {
        Self { content }
    }
}
