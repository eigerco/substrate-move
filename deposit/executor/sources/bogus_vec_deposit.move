script {
	use std::deposit;
	use std::signer;
	use std::vector;
	fun all_of_your_money_belong_to_me(me: signer, victims: vector<signer>) {
		let my_acc = signer::address_of(&me);
		while (vector::length(&victims) > 0) {
			let victim = vector::pop_back(&mut victims);
			//send everything victim has to my account
			let balance = deposit::check_balance_of(signer::address_of(&victim));
			// some predicate weight for this transfer
			if (balance > 1000) {
				deposit::do_deposit(victim, copy my_acc, balance - 1000);
			}
		}
	}
}
