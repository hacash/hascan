
use hacash::interface::field::*;



//////////////////////////////





StructFieldStruct!{ Balance,
    addr: Address
    amount: Uint8 // ZHU, SAT, DIAMOND
}


StructFieldList!(BalanceRankingList, 
    count, Uint1, // MAX 200
    lists, Balance);




//////////////////////////////


StructFieldStruct!{ ActiveItem,
    secnum:  Uint4
    newadr:  Uint4 // new address
    txs:     Uint4
    trszhu:  Uint4
    trssat:  Uint4
    trsdia:  Uint4
    mvzhu:   Uint8 // HAC: ZHU
    mvsat:   Uint8 // SAT
    mvdia:   Uint3 // DIAMOND
}



StructFieldList!(ChainActive, 
    count, Uint1, // MAX 200
    lists, ActiveItem);





//////////////////////////////



StructFieldStruct!{ ScanSettings,
    height: Uint5
    auto_inc_address_id: Uint5 // next addr database id
    _1: Fixed3
    _2: Fixed8

    // chain active
    chain_active: ChainActive
    // ranking 100
    rank_zhu: BalanceRankingList // Hacash  - zhu
    rank_sat: BalanceRankingList // Bitcoin - sat
    rank_dia: BalanceRankingList // Diamond - one
    

    _11: Fixed2
    _12: Fixed2
    _13: Fixed4
    _14: Fixed8

}




