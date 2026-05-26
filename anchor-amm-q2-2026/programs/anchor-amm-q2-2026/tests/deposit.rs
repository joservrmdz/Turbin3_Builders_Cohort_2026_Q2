mod common;
use common::*;

#[test]
fn test_deposit() {
    let mut t = AmmTest::initialized();
    t.deposit();

    assert_eq!(token_balance(&t.svm, &t.vault_x), DEPOSIT_X);
    assert_eq!(token_balance(&t.svm, &t.vault_y), DEPOSIT_Y);
    assert_eq!(token_balance(&t.svm, &t.user_ata_lp), LP_AMOUNT);
}
