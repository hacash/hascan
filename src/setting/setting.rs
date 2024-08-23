
use hacash::interface::field::*;



//////////////////////////////





StructFieldStruct!{ Balance,
    address: Address
    amount: Uint8 // ZHU, SAT, DIAMOND
}


StructFieldList!(BalanceRanking, 
    count, Uint1, // MAX 200
    lists, Balance);




//////////////////////////////



StructFieldStruct!{ ScanSettings,
    height: Uint5
    auto_inc_address_id: Uint5 // next addr database id
    _1: Fixed3
    _2: Fixed8
    _3: Fixed8
    _4: Fixed8
    _5: Fixed16
    _6: Fixed16
    _7: Fixed16

    // ranking
    rank_zhu: BalanceRanking // Hacash  - zhu
    rank_sat: BalanceRanking // Bitcoin - sat
    rank_dia: BalanceRanking // Diamond - one
    

    _11: Fixed2
    _12: Fixed2
    _13: Fixed4
    _14: Fixed8

    _16: Fixed16

    _27: Fixed32
    _28: Fixed32
    _29: Fixed32
    _30: Fixed32
    _31: Fixed32
    _32: Fixed32
}




