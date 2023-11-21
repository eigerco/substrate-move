address 0x1 {
	module deposit {
		use std::signer;

		/// Deposit structure to transfer or fetch own balance
		/// Bridge between Move and Substrate
		struct Deposit has key {
			destination: address,
			amount: u128
		}

		/// Public constructor - means to instantiate `Deposit`
		public fun new(destination: address, amount: u128): Deposit {
			Deposit { destination, amount }
		}

		/// Transfer 'amount' from given `Deposit` to given 'destination'
		/// Checks own balance prior to submitting the transaction
		public fun do_deposit(account: signer, destination: address, amount: u128) {
			//let current_balance = check_balance(account);
			//assert!(current_balance >= deposit.amount, 1);
			move_to(&account, Deposit { destination, amount } );
		}

		/// Check current balance of self by getting a deposit with own address
		public fun check_balance(account: &signer): u128 acquires Deposit {
			let my_acc = signer::address_of(account); 
			let balance = borrow_global<Deposit>(my_acc);
			assert!(balance.destination == my_acc, 2);
			balance.amount
		}

		/// Check current balance of `Deposit::destination` by getting a deposit with their address
		public fun check_balance_of(of: Deposit): u128 acquires Deposit {
			let Deposit { destination: of, amount: _ } = of;
			let Deposit { destination: _ , amount: balance } = borrow_global<Deposit>(of);
			*balance
		}

		#[test(s= @0x1234, d= @0xCAFE, f= @0x1234)]
		fun deposit_storage_test(s: signer, d: signer, f: signer) {
		    do_deposit(s, signer::address_of(&d), 1234);
		    assert!(exists<Deposit>(signer::address_of(&f)), 0);
		    assert!(!exists<Deposit>(signer::address_of(&d)), 1);
		}
	}
}
