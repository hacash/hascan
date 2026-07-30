pub fn update_ranking(
    setting: &mut ScanSettings,
    balances: impl IntoIterator<Item = (Address, Option<field::Balance>)>,
) -> Rerr {
    for (address, balance) in balances {
        let balance = balance.unwrap_or_default();
        let zhu = balance.hacash.to_zhu_u64()?;
        update_one_rank(&mut setting.rank_zhu, &address, zhu);
        update_one_rank(&mut setting.rank_sat, &address, balance.satoshi.uint());
        update_one_rank(&mut setting.rank_dia, &address, balance.diamond.uint());
    }

    truncate_rank(&mut setting.rank_zhu);
    truncate_rank(&mut setting.rank_sat);
    truncate_rank(&mut setting.rank_dia);
    Ok(())
}

fn truncate_rank(ranking: &mut BalanceRankingList) {
    ranking.lists.truncate(200);
    ranking.count = Uint1::from(ranking.lists.len() as u8);
}

fn update_one_rank(ranking: &mut BalanceRankingList, address: &Address, amount: u64) {
    let list = &mut ranking.lists;
    list.retain(|item| item.addr != *address);
    if amount == 0 {
        return;
    }

    let item = RankBalance {
        addr: *address,
        amount: Uint8::from(amount),
    };
    let position = list
        .iter()
        .position(|existing| amount > existing.amount.uint())
        .unwrap_or(list.len());
    list.insert(position, item);
}
