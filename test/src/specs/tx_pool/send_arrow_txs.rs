use crate::{Net, Spec, DEFAULT_TX_PROPOSAL_WINDOW};
use ckb_app_config::CKBAppConfig;
use ckb_tx_pool::FeeRate;
use ckb_types::{
    packed::{CellInput, OutPoint},
    prelude::*,
};

pub struct SendArrowTxs;

const MAX_ANCESTORS_COUNT: usize = 25;

impl Spec for SendArrowTxs {
    crate::name!("send_arrow_txs");

    fn run(&self, net: &mut Net) {
        let node0 = &net.nodes[0];

        node0.generate_blocks((DEFAULT_TX_PROPOSAL_WINDOW.1 + 2) as usize);
        // build arrow txs
        let mut txs = vec![node0.new_transaction_spend_tip_cellbase()];
        while txs.len() < MAX_ANCESTORS_COUNT + 1 {
            let parent = txs.last().unwrap();
            let child = parent
                .as_advanced_builder()
                .set_inputs(vec![{
                    CellInput::new_builder()
                        .previous_output(OutPoint::new(parent.hash(), 0))
                        .build()
                }])
                .set_outputs(vec![parent.output(0).unwrap()])
                .build();
            txs.push(child);
        }
        assert_eq!(txs.len(), MAX_ANCESTORS_COUNT + 1);
        // send arrow txs
        for tx in txs[..MAX_ANCESTORS_COUNT].iter() {
            node0.rpc_client().send_transaction(tx.data().into());
        }
        let ret = node0
            .rpc_client()
            .send_transaction_result(txs.last().unwrap().data().into());
        assert!(ret.is_err());
    }

    fn modify_ckb_config(&self) -> Box<dyn Fn(&mut CKBAppConfig) -> ()> {
        Box::new(|config| {
            config.tx_pool.min_fee_rate = FeeRate::from_u64(0);
            config.tx_pool.max_ancestors_count = MAX_ANCESTORS_COUNT;
        })
    }
}
