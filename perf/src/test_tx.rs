use {
    rand::{CryptoRng, Rng, RngCore},
    safecoin_sdk::{
        clock::Slot,
        hash::Hash,
        instruction::CompiledInstruction,
        signature::{Keypair, Signer},
        stake,
        system_instruction::SystemInstruction,
        system_program, system_transaction,
        transaction::Transaction,
    },
    solana_vote_program::vote_transaction,
};

pub fn test_tx() -> Transaction {
    let keypair1 = Keypair::new();
    let pubkey1 = keypair1.pubkey();
    let zero = Hash::default();
    system_transaction::transfer(&keypair1, &pubkey1, 42, zero)
}

pub fn test_invalid_tx() -> Transaction {
    let mut tx = test_tx();
    tx.signatures = vec![Transaction::get_invalid_signature()];
    tx
}

pub fn test_multisig_tx() -> Transaction {
    let keypair0 = Keypair::new();
    let keypair1 = Keypair::new();
    let keypairs = vec![&keypair0, &keypair1];
    let lamports = 5;
    let blockhash = Hash::default();

    let transfer_instruction = SystemInstruction::Transfer { lamports };

    let program_ids = vec![system_program::id(), stake::program::id()];

    let instructions = vec![CompiledInstruction::new(
        0,
        &transfer_instruction,
        vec![0, 1],
    )];

    Transaction::new_with_compiled_instructions(
        &keypairs,
        &[],
        blockhash,
        program_ids,
        instructions,
    )
}

pub fn new_test_vote_tx<R>(rng: &mut R) -> Transaction
where
    R: CryptoRng + RngCore,
{
    let mut slots: Vec<Slot> = std::iter::repeat_with(|| rng.gen()).take(5).collect();
    slots.sort_unstable();
    slots.dedup();
    let switch_proof_hash = rng.gen_bool(0.5).then(|| safecoin_sdk::hash::new_rand(rng));
    vote_transaction::new_vote_transaction(
        slots,
        safecoin_sdk::hash::new_rand(rng), // bank_hash
        safecoin_sdk::hash::new_rand(rng), // blockhash
        &Keypair::generate(rng),         // node_keypair
        &Keypair::generate(rng),         // vote_keypair
        &Keypair::generate(rng),         // authorized_voter_keypair
        switch_proof_hash,
    )
}
