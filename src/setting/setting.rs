use field::{Decode, Encode, Reader};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RankBalance {
    pub addr: Address,
    pub amount: Uint8,
}

impl Encode for RankBalance {
    fn size(&self) -> usize {
        self.addr.size() + self.amount.size()
    }
    fn encode_to(&self, out: &mut Vec<u8>) {
        self.addr.encode_to(out);
        self.amount.encode_to(out);
    }
}

impl Decode for RankBalance {
    fn decode(buf: &[u8]) -> Ret<(Self, usize)> {
        let mut reader = Reader::new(buf);
        let value = Self {
            addr: reader.read()?,
            amount: reader.read()?,
        };
        Ok((value, reader.used()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BalanceRankingList {
    pub count: Uint1,
    pub lists: Vec<RankBalance>,
}

impl BalanceRankingList {
    pub fn as_list(&self) -> &[RankBalance] {
        &self.lists
    }
}

impl Encode for BalanceRankingList {
    fn size(&self) -> usize {
        self.count.size() + self.lists.iter().map(Encode::size).sum::<usize>()
    }
    fn encode_to(&self, out: &mut Vec<u8>) {
        Uint1::from(self.lists.len() as u8).encode_to(out);
        for item in &self.lists {
            item.encode_to(out);
        }
    }
}

impl Decode for BalanceRankingList {
    fn decode(buf: &[u8]) -> Ret<(Self, usize)> {
        let mut reader = Reader::new(buf);
        let count: Uint1 = reader.read()?;
        let mut lists = Vec::with_capacity(count.uint() as usize);
        for _ in 0..count.uint() {
            lists.push(reader.read()?);
        }
        Ok((Self { count, lists }, reader.used()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActiveItem {
    pub secnum: Uint4,
    pub newadr: Uint4,
    pub txs: Uint4,
    pub trszhu: Uint4,
    pub trssat: Uint4,
    pub trsdia: Uint4,
    pub mvzhu: Uint16,
    pub mvsat: Uint12,
    pub mvdia: Uint8,
}

impl Encode for ActiveItem {
    fn size(&self) -> usize {
        self.secnum.size()
            + self.newadr.size()
            + self.txs.size()
            + self.trszhu.size()
            + self.trssat.size()
            + self.trsdia.size()
            + self.mvzhu.size()
            + self.mvsat.size()
            + self.mvdia.size()
    }
    fn encode_to(&self, out: &mut Vec<u8>) {
        self.secnum.encode_to(out);
        self.newadr.encode_to(out);
        self.txs.encode_to(out);
        self.trszhu.encode_to(out);
        self.trssat.encode_to(out);
        self.trsdia.encode_to(out);
        self.mvzhu.encode_to(out);
        self.mvsat.encode_to(out);
        self.mvdia.encode_to(out);
    }
}

impl Decode for ActiveItem {
    fn decode(buf: &[u8]) -> Ret<(Self, usize)> {
        let mut reader = Reader::new(buf);
        let value = Self {
            secnum: reader.read()?,
            newadr: reader.read()?,
            txs: reader.read()?,
            trszhu: reader.read()?,
            trssat: reader.read()?,
            trsdia: reader.read()?,
            mvzhu: reader.read()?,
            mvsat: reader.read()?,
            mvdia: reader.read()?,
        };
        Ok((value, reader.used()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChainActiveList {
    pub count: Uint1,
    pub lists: Vec<ActiveItem>,
}

impl ChainActiveList {
    pub fn as_list(&self) -> &[ActiveItem] {
        &self.lists
    }
}

impl Encode for ChainActiveList {
    fn size(&self) -> usize {
        self.count.size() + self.lists.iter().map(Encode::size).sum::<usize>()
    }
    fn encode_to(&self, out: &mut Vec<u8>) {
        Uint1::from(self.lists.len() as u8).encode_to(out);
        for item in &self.lists {
            item.encode_to(out);
        }
    }
}

impl Decode for ChainActiveList {
    fn decode(buf: &[u8]) -> Ret<(Self, usize)> {
        let mut reader = Reader::new(buf);
        let count: Uint1 = reader.read()?;
        let mut lists = Vec::with_capacity(count.uint() as usize);
        for _ in 0..count.uint() {
            lists.push(reader.read()?);
        }
        Ok((Self { count, lists }, reader.used()))
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ScanSettings {
    pub height: Uint5,
    pub auto_inc_address_id: Uint5,
    pub _1: Fixed3,
    pub _2: Fixed8,
    pub chain_active: ChainActiveList,
    pub rank_zhu: BalanceRankingList,
    pub rank_sat: BalanceRankingList,
    pub rank_dia: BalanceRankingList,
    pub _11: Fixed2,
    pub _12: Fixed2,
    pub _13: Fixed4,
    pub _14: Fixed8,
}

impl Encode for ScanSettings {
    fn size(&self) -> usize {
        self.height.size()
            + self.auto_inc_address_id.size()
            + self._1.size()
            + self._2.size()
            + self.chain_active.size()
            + self.rank_zhu.size()
            + self.rank_sat.size()
            + self.rank_dia.size()
            + self._11.size()
            + self._12.size()
            + self._13.size()
            + self._14.size()
    }
    fn encode_to(&self, out: &mut Vec<u8>) {
        self.height.encode_to(out);
        self.auto_inc_address_id.encode_to(out);
        self._1.encode_to(out);
        self._2.encode_to(out);
        self.chain_active.encode_to(out);
        self.rank_zhu.encode_to(out);
        self.rank_sat.encode_to(out);
        self.rank_dia.encode_to(out);
        self._11.encode_to(out);
        self._12.encode_to(out);
        self._13.encode_to(out);
        self._14.encode_to(out);
    }
}

impl Decode for ScanSettings {
    fn decode(buf: &[u8]) -> Ret<(Self, usize)> {
        let mut reader = Reader::new(buf);
        let value = Self {
            height: reader.read()?,
            auto_inc_address_id: reader.read()?,
            _1: reader.read()?,
            _2: reader.read()?,
            chain_active: reader.read()?,
            rank_zhu: reader.read()?,
            rank_sat: reader.read()?,
            rank_dia: reader.read()?,
            _11: reader.read()?,
            _12: reader.read()?,
            _13: reader.read()?,
            _14: reader.read()?,
        };
        Ok((value, reader.used()))
    }
}
