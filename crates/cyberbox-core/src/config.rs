pub struct Config {
    pub container_name: String,
    pub image_tag: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub registry_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            container_name: "cyberbox-toolbox".to_string(),
            image_tag: "cyberbox-toolbox:latest".to_string(),
            ollama_url: "http://127.0.0.1:11434".to_string(),
            ollama_model: "llama3.1:8b-instruct-q4_K_M".to_string(),
            registry_path: "registry/tools.toml".to_string(),
        }
    }
}
