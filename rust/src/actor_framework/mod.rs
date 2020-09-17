pub trait Message
where
    Self: Send,
    Self::Response: Send,
{
    type Response;
}

pub trait Handler<T>
where
    Self: Actor,
    T: Message,
{
    fn handle(&self, msg: T) -> T::Response;
}

pub trait Actor {
    fn start_default()
    where
        Self: Default,
    {
        Self::default().start()
    }

    fn start(self) {
        tokio::spawn()
    }
}
