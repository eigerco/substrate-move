address 0x1 {
	module test_struct {
		struct Test {
			sig: signer
		}

		public fun new(s: signer): Test {
			Test { sig: s }
		}

		public fun as_signer(t: Test): signer {
			let Test { sig: s } = t;
			s
		}
	}
}
