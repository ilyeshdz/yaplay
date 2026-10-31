pub struct Configuration {
    pub selected_monitor_id: u32,
    pub notifications_enabled: bool,
    pub websocket_port: u16,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            selected_monitor_id: 0,
            notifications_enabled: true,
            websocket_port: 8080,
        }
    }
}
