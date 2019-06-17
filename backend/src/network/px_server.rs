pub trait PxServer {
    ///
    /// Start the listener and handle new connections
    ///
    fn start(&mut self, listen_address: &String, port: u16);
}
