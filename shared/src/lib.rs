use tungstenite::Message;

#[derive(Debug, Clone)]
pub enum Transaction {
    Add(String),
    Delete(String),
    Update(String),
}

#[derive(Debug)]
pub enum Operation { // TODO: rename to operation
    RequestState,
    ExecuteTransaction(Transaction),
}

impl From<Message> for Operation {
    fn from(orig: Message) -> Self {
        let data = orig.into_data();
        let op_code = data.clone()[0];

        return match op_code {
            0x1 => Operation::RequestState,
            0x2 => {
                let transaction_opcode = data.clone()[1];
                let key = bincode::deserialize(&data.clone()[2..]).unwrap();

                let transaction = match transaction_opcode {
                    0x0 => Transaction::Add(key),
                    0x1 => Transaction::Update(key),
                    0x2 => Transaction::Delete(key),
                    _ => panic!("Unknown transaction_opcode {transaction_opcode}"),
                };

                Operation::ExecuteTransaction(transaction)
            },
            _ => panic!("Unknown opcode {op_code}"),
        }
    }
}

impl From<Operation> for Message {
    fn from(orig: Operation) -> Self {
        let op_code_value: Vec<u8> = match orig {
            Operation::RequestState => vec![0x1],
            Operation::ExecuteTransaction(transaction) => {
                let (transaction_opcode, key): (u8, String) = match transaction {
                    Transaction::Add(key) => {
                        (0x0, key)
                    },
                    Transaction::Update(key) => {
                        (0x1, key)
                    },
                    Transaction::Delete(key) => {
                        (0x2, key)
                    },
                };

                let mut encoded_key: Vec<u8> = bincode::serialize(&key).unwrap();

                let mut op_code = vec![0x2];

                op_code.push(transaction_opcode);
                op_code.append(&mut encoded_key);

                op_code
            },
        };

        Message::Binary(op_code_value.into())
    }
}
