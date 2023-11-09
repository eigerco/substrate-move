address 0x42 {
	module DepositModule {
		use std::signer;

		/// Deposit structure to transfer or fetch own balance
		/// Bridge between Move and Substrate-Move
		struct Deposit has key {
			destination: address,
			amount: u128
		}

		/// Transfer 'amount' from given `Deposit` to given 'destination'
		/// Checks own balance prior to submitting the transaction
		public fun do_deposit(account: &signer, deposit: Deposit) acquires Deposit {
			let current_balance = check_balance(account);
			assert!(current_balance >= deposit.amount, 1);
			move_to(account, deposit)
		}

		/// Check current balance of self by getting a deposit with own address
		public fun check_balance(account: &signer): u128 acquires Deposit {
			let my_acc =signer::address_of(account); 
			let balance = borrow_global<Deposit>(my_acc);
			assert!(balance.destination == my_acc, 2);
			balance.amount
		}
	}
}
