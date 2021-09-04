// Copyright (c) Facebook Inc.
// SPDX-License-Identifier: Apache-2.0

use super::*;
use std::collections::BTreeMap;

#[test]
fn test_signed_values() {
    let mut authorities = BTreeMap::new();
    let key1 = get_key_pair();
    let key2 = get_key_pair();
    let key3 = get_key_pair();
    let name1 = key1.public();
    let name2 = key2.public();

    authorities.insert(name1, /* voting right */ 1);
    authorities.insert(name2, /* voting right */ 0);
    let committee = Committee::new(authorities);

    let transfer = Transfer {
        account_id: dbg_account(1),
        recipient: Address::FastPay(dbg_account(2)),
        amount: Amount::from(1),
        sequence_number: SequenceNumber::new(),
        user_data: UserData::default(),
    };
    let order = TransferOrder::new(transfer.clone(), &key1);
    let mut bad_order = TransferOrder::new(transfer, &key2);
    bad_order.owner = name1;

    let v = SignedTransferOrder::new(order.clone(), &key1);
    assert!(v.check(&committee).is_ok());

    let v = SignedTransferOrder::new(order.clone(), &key2);
    assert!(v.check(&committee).is_err());

    let v = SignedTransferOrder::new(order, &key3);
    assert!(v.check(&committee).is_err());

    let v = SignedTransferOrder::new(bad_order, &key1);
    assert!(v.check(&committee).is_err());
}

#[test]
fn test_certificates() {
    let key1 = get_key_pair();
    let key2 = get_key_pair();
    let key3 = get_key_pair();
    let name1 = key1.public();
    let name2 = key2.public();

    let mut authorities = BTreeMap::new();
    authorities.insert(name1, /* voting right */ 1);
    authorities.insert(name2, /* voting right */ 1);
    let committee = Committee::new(authorities);

    let transfer = Transfer {
        account_id: dbg_account(1),
        recipient: Address::FastPay(dbg_account(1)),
        amount: Amount::from(1),
        sequence_number: SequenceNumber::new(),
        user_data: UserData::default(),
    };
    let order = TransferOrder::new(transfer.clone(), &key1);
    let mut bad_order = TransferOrder::new(transfer, &key2);
    bad_order.owner = name1;

    let v1 = SignedTransferOrder::new(order.clone(), &key1);
    let v2 = SignedTransferOrder::new(order.clone(), &key2);
    let v3 = SignedTransferOrder::new(order.clone(), &key3);

    let mut builder = SignatureAggregator::try_new(order.clone(), &committee).unwrap();
    assert!(builder
        .append(v1.authority, v1.signature)
        .unwrap()
        .is_none());
    let mut c = builder.append(v2.authority, v2.signature).unwrap().unwrap();
    assert!(c.check(&committee).is_ok());
    c.signatures.pop();
    assert!(c.check(&committee).is_err());

    let mut builder = SignatureAggregator::try_new(order, &committee).unwrap();
    assert!(builder
        .append(v1.authority, v1.signature)
        .unwrap()
        .is_none());
    assert!(builder.append(v3.authority, v3.signature).is_err());

    assert!(SignatureAggregator::try_new(bad_order, &committee).is_err());
}
