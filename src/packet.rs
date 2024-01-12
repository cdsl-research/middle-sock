use dhcproto::v4::Message;

pub struct DHCPMessage(Message);

impl From<Message> for DHCPMessage {
    fn from(value: Message) -> Self {
        DHCPMessage(value)
    }
}
