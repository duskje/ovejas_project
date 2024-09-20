use tungstenite::Message;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[derive(Debug)]
pub enum OpCode {
    RequestState,
    ExecuteTransaction,
}

impl From<Message> for OpCode {
    fn from(orig: Message) -> Self {
        let op_code = orig.into_data()[0];

        match op_code {
            0x10 => return OpCode::RequestState,
            _ => panic!("Unknown opcode: {}", op_code),
        }
    }
}

impl From<OpCode> for Message {
    fn from(orig: OpCode) -> Self {
        let op_code_value = match orig {
            OpCode::RequestState => 0x10,
            _ => panic!("Unknown opcode: {:?}", orig),
        };

        Message::Binary(vec![op_code_value].into())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
