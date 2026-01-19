use yrs::updates::decoder::Decode;
use yrs::{Doc, GetString, ReadTxn, Text, Transact, Update};

#[tokio::test]
async fn test_crdt_delta_sync_applies_updates() {
    let doc_a = Doc::new();
    let text_a = doc_a.get_or_insert_text("body");

    {
        let mut txn = doc_a.transact_mut();
        text_a.insert(&mut txn, 0, "Hello");
    }

    let full_update = doc_a
        .transact()
        .encode_state_as_update_v1(&yrs::StateVector::default());

    let doc_b = Doc::new();
    {
        let update = Update::decode_v1(&full_update).expect("decode full update");
        let mut txn = doc_b.transact_mut();
        txn.apply_update(update);
    }

    let text_b = doc_b.get_or_insert_text("body");
    {
        let txn_b = doc_b.transact();
        assert_eq!(text_b.get_string(&txn_b), "Hello");
    }

    {
        let mut txn = doc_a.transact_mut();
        text_a.insert(&mut txn, 5, " World");
    }

    let state_b = {
        let txn = doc_b.transact();
        txn.state_vector()
    };
    let delta_update = {
        let txn = doc_a.transact();
        txn.encode_state_as_update_v1(&state_b)
    };

    {
        let update = Update::decode_v1(&delta_update).expect("decode delta update");
        let mut txn = doc_b.transact_mut();
        txn.apply_update(update);
    }

    {
        let txn_b = doc_b.transact();
        assert_eq!(text_b.get_string(&txn_b), "Hello World");
    }
}
