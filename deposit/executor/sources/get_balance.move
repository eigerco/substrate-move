script {
	use std::deposit;
	fun check_balance_of(of: address, expected: u128) {
		assert!(deposit::check_balance_of(of) == expected, 0);
	}
}
