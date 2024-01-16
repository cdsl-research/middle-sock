use dhcproto::v4::Message;

pub struct DHCPMessage(Message);

impl DHCPMessage {
    pub fn raw(&self) -> Message {
        self.0.clone()
    }
}

impl From<Message> for DHCPMessage {
    fn from(value: Message) -> Self {
        DHCPMessage(value)
    }
}
