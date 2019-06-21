use witnet_crypto::hash::Sha256;
use witnet_crypto::merkle::{merkle_tree_root as crypto_merkle_tree_root, sha256_concat};
use witnet_data_structures::chain::*;
use witnet_data_structures::transaction::*;

/// Function to calculate a merkle tree from a transaction vector
pub fn merkle_tree_root<T>(transactions: &[T]) -> Hash
where
    T: Hashable,
{
    let transactions_hashes: Vec<Sha256> = transactions
        .iter()
        .map(|x| match x.hash() {
            Hash::SHA256(x) => Sha256(x),
        })
        .collect();

    Hash::from(crypto_merkle_tree_root(&transactions_hashes))
}

fn build_merkle_tree(block_header: &mut BlockHeader, txns: &BlockTransactions) {
    let merkle_roots = BlockMerkleRoots {
        mint_hash: txns.mint.hash(),
        vt_hash_merkle_root: merkle_tree_root(&txns.value_transfer_txns),
        dr_hash_merkle_root: merkle_tree_root(&txns.data_request_txns),
        commit_hash_merkle_root: merkle_tree_root(&txns.commit_txns),
        reveal_hash_merkle_root: merkle_tree_root(&txns.reveal_txns),
        tally_hash_merkle_root: merkle_tree_root(&txns.tally_txns),
    };
    block_header.merkle_roots = merkle_roots;
}

fn h(left: Hash, right: Hash) -> Hash {
    let left = match left {
        Hash::SHA256(x) => Sha256(x),
    };
    let right = match right {
        Hash::SHA256(x) => Sha256(x),
    };
    sha256_concat(left, right).into()
}

fn example_block(txns: BlockTransactions) -> Block {
    let current_epoch = 1000;
    let last_block_hash = "62adde3e36db3f22774cc255215b2833575f66bf2204011f80c03d34c7c9ea41"
        .parse()
        .unwrap();

    let block_beacon = CheckpointBeacon {
        checkpoint: current_epoch,
        hash_prev_block: last_block_hash,
    };
    let mut block_header = BlockHeader::default();
    build_merkle_tree(&mut block_header, &txns);
    block_header.beacon = block_beacon;

    let block_sig = KeyedSignature::default();

    Block {
        block_header,
        block_sig,
        txns,
    }
}

fn example_dr(id: usize) -> DRTransaction {
    let dr_output = DataRequestOutput {
        value: id as u64,
        ..Default::default()
    };
    let dr_body = DRTransactionBody::new(vec![], vec![], dr_output);

    DRTransaction::new(dr_body, vec![])
}

#[test]
fn dr_inclusion_0_drs() {
    let block = example_block(BlockTransactions {
        data_request_txns: vec![],
        ..Default::default()
    });

    let dr = example_dr(0);
    assert_eq!(dr.proof_of_inclusion(&block), None);
}

#[test]
fn dr_inclusion_1_drs() {
    let drx = example_dr(0);
    let dr0 = example_dr(1);

    let block = example_block(BlockTransactions {
        data_request_txns: vec![dr0.clone()],
        ..Default::default()
    });

    assert_eq!(drx.proof_of_inclusion(&block), None);
    assert_eq!(
        dr0.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 0,
            lemma: vec![],
        })
    );
}

#[test]
fn dr_inclusion_2_drs() {
    let drx = example_dr(0);
    let dr0 = example_dr(1);
    let dr1 = example_dr(2);

    let block = example_block(BlockTransactions {
        data_request_txns: vec![dr0.clone(), dr1.clone()],
        ..Default::default()
    });

    assert_eq!(drx.proof_of_inclusion(&block), None);
    assert_eq!(
        dr0.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 0,
            lemma: vec![dr1.hash()],
        })
    );
    assert_eq!(
        dr1.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 1,
            lemma: vec![dr0.hash()],
        })
    );
}

#[test]
fn dr_inclusion_3_drs() {
    let drx = example_dr(0);
    let dr0 = example_dr(1);
    let dr1 = example_dr(2);
    let dr2 = example_dr(3);

    let block = example_block(BlockTransactions {
        data_request_txns: vec![dr0.clone(), dr1.clone(), dr2.clone()],
        ..Default::default()
    });

    assert_eq!(drx.proof_of_inclusion(&block), None);
    assert_eq!(
        dr0.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 0,
            lemma: vec![dr1.hash(), dr2.hash()],
        })
    );
    assert_eq!(
        dr1.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 1,
            lemma: vec![dr0.hash(), dr2.hash()],
        })
    );
    assert_eq!(
        dr2.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 1,
            lemma: vec![h(dr0.hash(), dr1.hash())],
        })
    );
}

#[test]
fn dr_inclusion_5_drs() {
    let drx = example_dr(0);
    let dr0 = example_dr(1);
    let dr1 = example_dr(2);
    let dr2 = example_dr(3);
    let dr3 = example_dr(4);
    let dr4 = example_dr(5);

    let block = example_block(BlockTransactions {
        data_request_txns: vec![
            dr0.clone(),
            dr1.clone(),
            dr2.clone(),
            dr3.clone(),
            dr4.clone(),
        ],
        ..Default::default()
    });

    assert_eq!(drx.proof_of_inclusion(&block), None);
    assert_eq!(
        dr0.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 0,
            lemma: vec![dr1.hash(), h(dr2.hash(), dr3.hash()), dr4.hash()],
        })
    );
    assert_eq!(
        dr1.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 1,
            lemma: vec![dr0.hash(), h(dr2.hash(), dr3.hash()), dr4.hash()],
        })
    );
    assert_eq!(
        dr2.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 2,
            lemma: vec![dr3.hash(), h(dr0.hash(), dr1.hash()), dr4.hash()],
        })
    );
    assert_eq!(
        dr3.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 3,
            lemma: vec![dr2.hash(), h(dr0.hash(), dr1.hash()), dr4.hash()],
        })
    );
    assert_eq!(
        dr4.proof_of_inclusion(&block),
        Some(TxInclusionProof {
            index: 1,
            lemma: vec![h(h(dr0.hash(), dr1.hash()), h(dr2.hash(), dr3.hash()))],
        })
    );
}