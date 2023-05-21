use aqueous::message::*;

pub fn main() {
    let deposit = Deposit { amount: 10 };
    let message = Msg { data: deposit };

    message_handler(message);
}

pub struct Deposit {
    pub amount: u64,
}
impl Message for Deposit {}

pub fn message_handler(deposit: Msg<Deposit>) {
    let amount = deposit.data.amount;
    println!("Deposit: {}", amount);
}
