script {
    use 0x01::DepositModule::Deposit;

    fun transfer(me: signer, destination: address, amount: u128) {
        let destination: address = @0101;
        let d = Deposit::new(destination, amount);
        let result = Deposit::do_deposit(d, me); 
    }
}