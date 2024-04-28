use ton_types::{deserialize_cells_tree, HashmapE, HashmapType, SliceData, UInt256, Result};
use ton_block::{ConfigParamEnum, ConfigParams, Serializable};

#[cfg(test)]
pub(crate) fn dump_config(params: &HashmapE) {
    params.iterate_slices(|ref mut key, ref mut slice| -> Result<bool> {
        let key = key.get_next_u32()?;
        match ConfigParamEnum::construct_from_slice_and_number(&mut SliceData::load_cell(slice.reference(0)?)?, key)? {
            ConfigParamEnum::ConfigParam31(ref mut cfg) => {
                println!("\tConfigParam31.fundamental_smc_addr");
                cfg.fundamental_smc_addr.iterate_keys(|addr: UInt256| -> Result<bool> {
                    println!("\t\t{}", addr);
                    Ok(true)
                })?;
            }
            ConfigParamEnum::ConfigParam34(ref mut cfg) => {
                println!("\tConfigParam34.cur_validators");
                for validator in cfg.cur_validators.list() {
                    println!("\t\t{:?}", validator);
                };
            }
            x => println!("\t{:?}", x)
        }
        Ok(true)
    }).unwrap();
}

#[cfg(feature = "ton")]
#[test]
fn test_real_ton_config_params() {
    let mut bytes = std::fs::read("tests/data/config.boc").unwrap();
    let cell = deserialize_cells_tree(&mut &*bytes).unwrap().remove(0);
    let config1 = ConfigParams::with_address_and_params(UInt256::from([1; 32]), Some(cell));
    dump_config(&config1.config_params);
    assert!(!config1.valid_config_data(false, None).unwrap()); // fake config address
    assert!(config1.valid_config_data(true, None).unwrap());   // but other are ok
    let mut config2 = config1.clone();
    assert!(!config1.important_config_parameters_changed(&config2, true).unwrap());
    assert!(!config1.important_config_parameters_changed(&config2, false).unwrap());

    if let Some(ConfigParamEnum::ConfigParam0(param)) = config1.config(0).unwrap() {
        config2.config_addr = param.config_addr;
    }
    assert!(config2.valid_config_data(false, None).unwrap()); // real adress
    assert!(config2.valid_config_data(true, None).unwrap());

    assert!(!config1.important_config_parameters_changed(&config2, true).unwrap());
    assert!(!config1.important_config_parameters_changed(&config2, false).unwrap());

    if let Ok(Some(ConfigParamEnum::ConfigParam9(param))) = config1.config(9) {
        println!("Mandatory params indeces {:?}", param.mandatory_params.export_keys::<i32>());
    }
    if let Ok(Some(ConfigParamEnum::ConfigParam10(param))) = config1.config(10) {
        println!("Critical params indeces {:?}", param.critical_params.export_keys::<i32>());
    }
    //  remove mandatory parameter - make config not valid
    let key = SliceData::load_builder(14u32.write_to_new_cell().unwrap()).unwrap();
    config2.config_params.remove(key).unwrap();
    assert!(!config2.valid_config_data(true, None).unwrap());
}