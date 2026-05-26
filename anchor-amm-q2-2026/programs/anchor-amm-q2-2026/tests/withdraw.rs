mod common;
use common::*;

#[test]
fn test_withdraw() {
    let mut t = AmmTest::after_swap();
    t.withdraw();

    assert_eq!(token_balance(&t.svm, &t.vault_x), 0);
    assert_eq!(token_balance(&t.svm, &t.vault_y), 0);
    assert_eq!(token_balance(&t.svm, &t.user_ata_lp), 0);
}
