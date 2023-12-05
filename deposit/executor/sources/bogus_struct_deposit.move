script {
	use std::deposit;
	use std::signer;
	use std::test_struct;
	use std::test_struct::Test;
	fun bogus_struct(me: signer, victim: Test) {
		let my_acc = signer::address_of(&me);
		let sig = test_struct::as_signer(victim);
		let balance = deposit::check_balance_of(signer::address_of(&sig));
		// some predicate weight for this transfer
		if (balance > 1000) {
			deposit::do_deposit(sig, copy my_acc, balance - 1000);
		}
	}
}

