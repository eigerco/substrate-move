script {
    use std::string;
    use std::vector;

    fun magic_gas_costs_vector(_: signer) {
        // string::check_utf8
        let vec = vector::empty<u8>();
        vector::push_back(&mut vec, 65);
        vector::push_back(&mut vec, 66);
        vector::push_back(&mut vec, 67);
        let str = string::utf8(vec);
        assert!(string::length(&str) == 3, 5);
    }
}

script {
    use std::signer;

    fun magic_gas_costs_signer(who: signer) {
        // signer::address_of
        let addr = signer::address_of(&who);
        assert!(addr == @Bob, 0);
    }
}

script {
    use std::ascii;
    use std::type_name;

    fun magic_gas_costs_type_name1(_: signer) {
        // type_name::get
        let type = type_name::get<bool>();
        let type = type_name::into_string(type);
        assert!(ascii::length(&type) == 4, 1);
    }
}

script {
    use std::ascii;
    use std::type_name;
    use Bob::Module::ThisIsAVeryLongDatatypeNameForTesting;

    fun magic_gas_costs_type_name2(_: signer) {
        // type_name::get
        let type = type_name::get<ThisIsAVeryLongDatatypeNameForTesting>();
        let type = type_name::into_string(type);
        assert!(ascii::length(&type) != 38, 1);
    }
}

script {
    use substrate::balance;

    fun magic_gas_costs_balance_cheque_amount(_: signer) {
        // balance::transfer
        assert!(balance::cheque_amount(@Bob) > 0, 2);
    }
}

script {
    use substrate::balance;

    fun magic_gas_costs_balance_total_amount(_: signer) {
        // balance::transfer
        assert!(balance::total_amount(@Bob) > 0, 3);
    }
}

script {
    use substrate::balance;

    fun magic_gas_costs_balance_transfer(who: signer) {
        // balance::transfer
        assert!(balance::transfer(&who, @Alice, 10), 4);
    }
}
