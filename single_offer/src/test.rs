#![cfg(test)]
extern crate std;

use crate::{token, SingleOfferClient};
use soroban_sdk::{testutils::Address as _, Address, Env, IntoVal, Symbol};

fn create_token_contract<'a>(e: &Env, admin: &Address) -> token::Client<'a> {
    token::Client::new(e, &e.register_stellar_asset_contract(admin.clone()))
}

fn create_single_offer_contract<'a>(
    e: &Env,
    seller: &Address,
    sell_token: &Address,
    buy_token: &Address,
    sell_price: u32,
    buy_price: u32,
) -> SingleOfferClient<'a> {
    let offer = SingleOfferClient::new(e, &e.register_contract(None, crate::SingleOffer {}));
    offer.create(seller, sell_token, buy_token, &sell_price, &buy_price);

    // Verify that authorization is required for the seller.
    assert_eq!(
        e.auths(),
        [(
            seller.clone(),
            offer.address.clone(),
            Symbol::short("create"),
            (
                seller,
                sell_token.clone(),
                buy_token.clone(),
                sell_price,
                buy_price
            )
                .into_val(e)
        )]
    );

    offer
}

#[test]
fn test() {
    let e = Env::default();
    e.mock_all_auths();

    let token_admin = Address::random(&e);
    let seller = Address::random(&e);
    let buyer = Address::random(&e);
    let sell_token = create_token_contract(&e, &token_admin);
    let buy_token = create_token_contract(&e, &token_admin);

    // The price here is 1 sell_token for 2 buy_token.
    let offer =
        create_single_offer_contract(&e, &seller, &sell_token.address, &buy_token.address, 1, 2);
    // Give some sell_token to seller and buy_token to buyer.
    sell_token.mint(&seller, &1000);
    buy_token.mint(&buyer, &1000);
    // Deposit 100 sell_token from seller into offer.
    sell_token.transfer(&seller, &offer.address, &100);

    // Try trading 20 buy_token for at least 11 sell_token - that wouldn't
    // succeed because the offer price would result in 10 sell_token.
    assert!(offer.try_trade(&buyer, &20_i128, &11_i128).is_err());
    // Buyer trades 20 buy_token for 10 sell_token.
    offer.trade(&buyer, &20_i128, &10_i128);
    // Verify that authorization is required for the buyer.
    assert_eq!(
        e.auths(),
        [
            (
                buyer.clone(),
                offer.address.clone(),
                Symbol::short("trade"),
                (&buyer, 20_i128, 10_i128).into_val(&e)
            ),
            (
                buyer.clone(),
                buy_token.address.clone(),
                Symbol::new(&e, "transfer"),
                (buyer.clone(), &offer.address, 20_i128).into_val(&e)
            )
        ]
    );

    assert_eq!(sell_token.balance(&seller), 900);
    assert_eq!(sell_token.balance(&buyer), 10);
    assert_eq!(sell_token.balance(&offer.address), 90);
    assert_eq!(buy_token.balance(&seller), 20);
    assert_eq!(buy_token.balance(&buyer), 980);
    assert_eq!(buy_token.balance(&offer.address), 0);

    // Withdraw 70 sell_token from offer.
    offer.withdraw(&sell_token.address, &70);
    // Verify that the seller has to authorize this.
    assert_eq!(
        e.auths(),
        [(
            seller.clone(),
            offer.address.clone(),
            Symbol::short("withdraw"),
            (sell_token.address.clone(), 70_i128).into_val(&e)
        )]
    );

    assert_eq!(sell_token.balance(&seller), 970);
    assert_eq!(sell_token.balance(&offer.address), 20);

    // The price here is 1 sell_token = 1 buy_token.
    offer.updt_price(&1, &1);
    // Verify that the seller has to authorize this.
    assert_eq!(
        e.auths(),
        [(
            seller.clone(),
            offer.address.clone(),
            Symbol::new(&e, "updt_price"),
            (1_u32, 1_u32).into_val(&e)
        )]
    );

    // Buyer trades 10 buy_token for 10 sell_token.
    offer.trade(&buyer, &10_i128, &9_i128);
    assert_eq!(sell_token.balance(&seller), 970);
    assert_eq!(sell_token.balance(&buyer), 20);
    assert_eq!(sell_token.balance(&offer.address), 10);
    assert_eq!(buy_token.balance(&seller), 30);
    assert_eq!(buy_token.balance(&buyer), 970);
    assert_eq!(buy_token.balance(&offer.address), 0);
}
