pub fn record_ecosystem_action(
    dbtx: &mut DBTransaction,
    adrs: &mut AddressCache,
    setting: &mut ScanSettings,
    act: &dyn Action,
    main_addr: &Address,
    main_aid: u64,
    height: u64,
    blkts: u64,
) -> DBResult<bool> {
    if act.kind() == AssetCreate::KIND {
        let action = act
            .as_any()
            .downcast_ref::<AssetCreate>()
            .ok_or_else(|| db_fault("hascan AssetCreate type mismatch"))?;
        let metadata = &action.metadata;
        let issuer_aid = record_addr_id(dbtx, adrs, setting, &metadata.issuer, blkts)?;
        let ticket = String::from_utf8_lossy(&metadata.ticket.to_vec()).into_owned();
        let name = String::from_utf8_lossy(&metadata.name.to_vec()).into_owned();
        dbtx.execute(
            "INSERT OR IGNORE INTO asset_create \
             (serial,height,issuer_aid,ticket,name,decimal,supply,protocol_cost) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            (
                metadata.serial.uint(),
                height,
                issuer_aid,
                ticket,
                name,
                metadata.decimal.uint(),
                metadata.supply.uint().to_string(),
                action.protocol_cost.to_fin_string(),
            ),
        )?;
        return Ok(true);
    }

    if act.kind() == ContractDeploy::KIND {
        let action = act
            .as_any()
            .downcast_ref::<ContractDeploy>()
            .ok_or_else(|| db_fault("hascan ContractDeploy type mismatch"))?;
        let address = ContractAddress::calculate(main_addr, &action.nonce).to_readable();
        dbtx.execute(
            "INSERT OR IGNORE INTO contract_deploy \
             (address,height,deployer_aid,nonce,code_size,protocol_cost) \
             VALUES (?1,?2,?3,?4,?5,?6)",
            (
                address,
                height,
                main_aid,
                action.nonce.uint(),
                action.contract.size() as u64,
                action.protocol_cost.to_fin_string(),
            ),
        )?;
        return Ok(true);
    }

    if act.kind() == ContractUpdate::KIND {
        let action = act
            .as_any()
            .downcast_ref::<ContractUpdate>()
            .ok_or_else(|| db_fault("hascan ContractUpdate type mismatch"))?;
        dbtx.execute(
            "INSERT INTO contract_update (address,height,updater_aid,edit_size,protocol_cost) \
             VALUES (?1,?2,?3,?4,?5)",
            (
                action.address.to_readable(),
                height,
                main_aid,
                action.edit.size() as u64,
                action.protocol_cost.to_fin_string(),
            ),
        )?;
        return Ok(true);
    }

    Ok(false)
}
