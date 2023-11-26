use xcm_playground::*;
use xcm::prelude::*;
use frame_support::assert_ok;
use test_log::test;

#[test]
fn simple_execute() {
    ParaA::execute_with(|| {
        // ALICE starts with INITIAL_BALANCE, BOB starts with 0
        assert_eq!(parachain::Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(parachain::Balances::free_balance(BOB), 0);

        let amount_to_transfer = 100u128;
        let message = Xcm::<()>(vec![
            WithdrawAsset((Parent, amount_to_transfer).into()),
            BuyExecution { fees: (Parent, amount_to_transfer).into(), weight_limit: Unlimited },
            DepositAsset { assets: All.into(), beneficiary: AccountId32 { id: BOB.into(), network: None }.into() },
        ]);

        assert_ok!(parachain::PolkadotXcm::execute(
            parachain::RuntimeOrigin::signed(ALICE),
            Box::new(VersionedXcm::V3(message.into())),
            Weight::from_parts(100_000_000_000, 100_000_000_000),
        ));

        // After executing the message, balances have changed
        assert_eq!(parachain::Balances::free_balance(ALICE), INITIAL_BALANCE - amount_to_transfer);
        assert_eq!(parachain::Balances::free_balance(BOB), amount_to_transfer);
    });
}

#[test]
fn simple_send() {
    let amount_to_transfer = 100u128;

    ParaB::execute_with(|| {
        // ALICE's sovereign account starts at INITIAL_BALANCE
        assert_eq!(parachain::Balances::free_balance(sibling_account_account_id(1, ALICE)), INITIAL_BALANCE);
        // The BOB in this chain, alter-BOB, has no balance either
        assert_eq!(parachain::Balances::free_balance(BOB), 0);
    });

    ParaA::execute_with(|| {
        // ALICE starts with INITIAL_BALANCE, the BOB in this chain also starts with 0
        assert_eq!(parachain::Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(parachain::Balances::free_balance(BOB), 0);

        let message = Xcm::<()>(vec![
            WithdrawAsset((Parent, amount_to_transfer).into()),
            BuyExecution { fees: (Parent, amount_to_transfer).into(), weight_limit: Unlimited },
            DepositAsset { assets: All.into(), beneficiary: AccountId32 { id: BOB.into(), network: None }.into() },
        ]);

        assert_ok!(parachain::PolkadotXcm::send(
            parachain::RuntimeOrigin::signed(ALICE),
            Box::new(VersionedMultiLocation::V3((Parent, Parachain(2)).into())),
            Box::new(VersionedXcm::V3(message.into())),
        ));

        // After sending the message, nothing changed locally
        assert_eq!(parachain::Balances::free_balance(ALICE), INITIAL_BALANCE);
        assert_eq!(parachain::Balances::free_balance(BOB), 0);
    });

    ParaB::execute_with(|| {
        // Message was executed here and the balances of the sovereign account of Alice changes
        assert_eq!(parachain::Balances::free_balance(sibling_account_account_id(1, ALICE)), INITIAL_BALANCE - amount_to_transfer);
        // This is another account called BOB in ParaB, no the original bob and not even his sovereign account
        assert_eq!(parachain::Balances::free_balance(BOB), amount_to_transfer);
    });
}
