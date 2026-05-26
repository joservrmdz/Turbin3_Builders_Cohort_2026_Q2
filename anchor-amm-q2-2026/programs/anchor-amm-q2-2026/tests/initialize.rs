mod common;
use common::*;
use anchor_lang::AccountDeserialize;
use anchor_amm_q2_2026::state::Config;

#[test]
fn test_is_initialized() {
    let mut t = AmmTest::new();
    t.initialize();

    let raw = t.svm.get_account(&t.config).unwrap();
    let cfg = Config::try_deserialize(&mut raw.data.as_ref()).unwrap();
    assert_eq!(cfg.mint_x, t.mint_x);
    assert_eq!(cfg.mint_y, t.mint_y);
    assert_eq!(cfg.authority, None);
    assert_eq!(cfg.seed, SEED);
}
