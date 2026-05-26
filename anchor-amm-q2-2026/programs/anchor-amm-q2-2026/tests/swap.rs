mod common;
use common::*;

#[test]
fn test_swap() {
    let mut t = AmmTest::with_liquidity();
    t.swap();

    assert_eq!(token_balance(&t.svm, &t.vault_x), VAULT_X_AFTER_SWAP);
    assert_eq!(token_balance(&t.svm, &t.vault_y), VAULT_Y_AFTER_SWAP);
}
