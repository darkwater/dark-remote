use dark_remote_common::config::RemoteCommand;
use ewebsock::WsEvent;

pub struct Connection {
    sender: ewebsock::WsSender,
    receiver: ewebsock::WsReceiver,
}

impl Connection {
    pub fn new(url: &str) -> Result<Self, ewebsock::Error> {
        let options = ewebsock::Options::default();
        let (sender, receiver) = ewebsock::connect(url, options)?;

        Ok(Self { sender, receiver })
    }

    pub fn check_msg(&self) -> Option<String> {
        Some(match self.receiver.try_recv()? {
            WsEvent::Opened => "Connection opened".to_owned(),
            WsEvent::Message(_) => "Received a message".to_owned(),
            WsEvent::Error(e) => format!("Connection error: {e}"),
            WsEvent::Closed => None?,
        })
    }

    pub fn send(&mut self, cmd: RemoteCommand) {
        let msg = serde_json::to_string(&cmd).expect("Failed to serialize RemoteCommand");
        self.sender.send(ewebsock::WsMessage::Text(msg));
    }
}
