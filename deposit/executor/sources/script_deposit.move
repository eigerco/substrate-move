script {
    use std::deposit;

    fun transfer(me: signer, destination: address, amount: u128) {
        deposit::do_deposit(me, destination, amount)
    }
}
