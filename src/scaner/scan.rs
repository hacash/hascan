use std::collections::HashSet;

fn validate_next_height(checkpoint: u64, height: u64) -> Rerr {
    let expected = checkpoint.saturating_add(1);
    if height < expected {
        return Ok(());
    }
    if height != expected {
        return sys::errf!(
            "hascan block gap: expected height {}, received {}",
            expected,
            height
        );
    }
    Ok(())
}

fn do_scan(
    setting: &mut ScanSettings,
    dbtx: &mut rusqlite::Transaction<'_>,
    addresses: &mut AddressCache,
    block: &dyn Block,
) -> Rerr {
    let height = block.height();
    let expected = setting.height.uint().saturating_add(1);
    if height < expected {
        return Ok(());
    }
    validate_next_height(setting.height.uint(), height)?;
    if height.is_multiple_of(1000) {
        println!("[hascan] indexed block height {}", height);
    }

    let address_count_before = setting.auto_inc_address_id.uint();
    let timestamp = block.timestamp();

    let prelude = block.prelude_transaction()?;
    if let Some(miner) = prelude.author() {
        let (_, account) = record_addr_as_mut(dbtx, addresses, setting, &miner, timestamp)
            .map_err(|e| sys::Error::fault(e.to_string()))?;
        if let Some(reward) = prelude.block_reward() {
            let reward = reward.to_mei_u64()?;
            account.block_reward = account
                .block_reward
                .checked_add(reward)
                .ok_or_else(|| sys::Error::fault("hascan block reward overflow"))?;
        }
    }

    let transactions = block.transactions();
    let transaction_count = u32::try_from(transactions.len().saturating_sub(1))
        .map_err(|_| sys::Error::fault("hascan transaction count overflow"))?;
    let active = record_current_active(setting, height);
    active.txs = Uint4::from(
        active
            .txs
            .uint()
            .checked_add(transaction_count)
            .ok_or_else(|| sys::Error::fault("hascan transaction count overflow"))?,
    );
    for transaction in transactions.iter().skip(1) {
        record_coin_transfer(
            dbtx,
            addresses,
            transaction.as_ref(),
            setting,
            height,
            timestamp,
        )
        .map_err(|e| sys::Error::fault(e.to_string()))?;
    }

    let newadr = setting
        .auto_inc_address_id
        .uint()
        .checked_sub(address_count_before)
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| sys::Error::fault("hascan new address count overflow"))?;
    update_chain_active(setting, newadr, height)?;
    setting.height = Uint5::from(height);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_next_height;

    #[test]
    fn rejects_block_gaps() {
        assert!(validate_next_height(42, 43).is_ok());
        assert!(validate_next_height(42, 44).is_err());
        // Replayed blocks are harmless; do_scan returns before writing them.
        assert!(validate_next_height(42, 42).is_ok());
    }
}

impl BlkScaner {
    fn catch_up(&self, view: Arc<dyn ScanerView>, rebuild_ranking: bool) -> Rerr {
        let _serial = self.inner.catchup.lock().unwrap();
        let mut touched = HashSet::new();

        loop {
            let history = view.block_history();
            let stable_height = history.stable_height();
            let target = if self.cnf.stop_at_height == 0 {
                stable_height
            } else {
                stable_height.min(self.cnf.stop_at_height)
            };
            let current = self.inner.setting.lock().unwrap().height.uint();
            if current >= target {
                break;
            }

            let batch_end = current
                .saturating_add(self.cnf.scan_batch_blocks)
                .min(target);
            let mut dbconn = self.inner.dbconn.lock().unwrap();
            let mut setting = self.inner.setting.lock().unwrap();
            let mut next_setting = setting.clone();
            let mut addresses = AddressCache::new();
            let mut dbtx = dbconn
                .transaction()
                .map_err(|e| sys::Error::fault(e.to_string()))?;

            for height in current + 1..=batch_end {
                let block = history.block_at_height(height).ok_or_else(|| {
                    sys::Error::fault(format!("hascan cannot load stable block {}", height))
                })?;
                do_scan(&mut next_setting, &mut dbtx, &mut addresses, block.as_ref())?;
            }
            insert_update_addr(&mut dbtx, &addresses)
                .map_err(|e| sys::Error::fault(e.to_string()))?;
            save_scan_height_tx(&mut dbtx, next_setting.height.uint())
                .map_err(|e| sys::Error::fault(e.to_string()))?;
            save_scan_settings_tx(&mut dbtx, &next_setting.encode())
                .map_err(|e| sys::Error::fault(e.to_string()))?;
            dbtx.commit()
                .map_err(|e| sys::Error::fault(e.to_string()))?;
            *setting = next_setting;

            if !rebuild_ranking {
                for address in addresses.keys() {
                    touched.insert(Address::from_readable(address)?);
                }
            }
        }

        let addresses = if rebuild_ranking {
            let dbconn = self.inner.dbconn.lock().unwrap();
            load_all_account_addresses(&dbconn).map_err(|e| sys::Error::fault(e.to_string()))?
        } else {
            touched.into_iter().collect()
        };
        self.refresh_ranking(view.as_ref(), addresses)?;
        let setting = self.inner.setting.lock().unwrap();
        if let Err(e) = crate::save_setting(&self.cnf.datadir, &setting) {
            eprintln!("[hascan] settings mirror write failed: {e}");
        }
        Ok(())
    }

    fn refresh_ranking(&self, view: &dyn ScanerView, addresses: Vec<Address>) -> Rerr {
        if addresses.is_empty() {
            return Ok(());
        }
        let history = view.block_history();
        let height = history.stable_height();
        let block = history.block_at_height(height).ok_or_else(|| {
            sys::Error::fault(format!(
                "hascan cannot load ranking snapshot block {}",
                height
            ))
        })?;
        let balances = view
            .balances_at(&block.hash(), &addresses)
            .ok_or_else(|| sys::Error::fault("hascan ranking snapshot unavailable"))?;

        let mut dbconn = self.inner.dbconn.lock().unwrap();
        let mut setting = self.inner.setting.lock().unwrap();
        let mut next_setting = setting.clone();
        update_ranking(&mut next_setting, addresses.into_iter().zip(balances))?;
        let data = next_setting.encode();
        let setting_height = next_setting.height.uint();
        save_scan_settings_conn(&mut dbconn, setting_height, &data)
            .map_err(|e| sys::Error::fault(e.to_string()))?;
        *setting = next_setting;
        Ok(())
    }
}
