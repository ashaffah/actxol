pub struct AppState {
    pub postgres_db: Box<dyn ConnectPosgresDB>,
}

pub trait ConnectPosgresDB {
    fn connect(&self) -> Result<(), Box<dyn std::error::Error>>;
    fn get_connection_string(&self) -> &str;
}
