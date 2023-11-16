script {
    use std::deposit;

    fun transfer(me: signer, destination: address, amount: u128) {
        let d = deposit::new(destination, amount);
        deposit::do_deposit(d, &me);
    }
}