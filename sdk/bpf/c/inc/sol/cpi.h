#pragma once
/**
 * @brief Safecoin Cross-Program Invocation
 */

#include <sol/types.h>
#include <sol/pubkey.h>
#include <sol/entrypoint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Account Meta
 */
typedef struct {
  SafePubkey *pubkey; /** An account's public key */
  bool is_writable; /** True if the `pubkey` can be loaded as a read-write account */
  bool is_signer; /** True if an Instruction requires a Transaction signature matching `pubkey` */
} SafeAccountMeta;

/**
 * Instruction
 */
typedef struct {
  SafePubkey *program_id; /** Pubkey of the instruction processor that executes this instruction */
  SafeAccountMeta *accounts; /** Metadata for what accounts should be passed to the instruction processor */
  uint64_t account_len; /** Number of SafeAccountMetas */
  uint8_t *data; /** Opaque data passed to the instruction processor */
  uint64_t data_len; /** Length of the data in bytes */
} SafeInstruction;

/**
 * Internal cross-program invocation function
 */
uint64_t sol_invoke_signed_c(
  const SafeInstruction *instruction,
  const SafeAccountInfo *account_infos,
  int account_infos_len,
  const SafeSignerSeeds *signers_seeds,
  int signers_seeds_len
);

/**
 * Invoke another program and sign for some of the keys
 *
 * @param instruction Instruction to process
 * @param account_infos Accounts used by instruction
 * @param account_infos_len Length of account_infos array
 * @param seeds Seed bytes used to sign program accounts
 * @param seeds_len Length of the seeds array
 */
static uint64_t sol_invoke_signed(
    const SafeInstruction *instruction,
    const SafeAccountInfo *account_infos,
    int account_infos_len,
    const SafeSignerSeeds *signers_seeds,
    int signers_seeds_len
) {
  return sol_invoke_signed_c(
    instruction,
    account_infos,
    account_infos_len,
    signers_seeds,
    signers_seeds_len
  );
}
/**
 * Invoke another program
 *
 * @param instruction Instruction to process
 * @param account_infos Accounts used by instruction
 * @param account_infos_len Length of account_infos array
*/
static uint64_t sol_invoke(
    const SafeInstruction *instruction,
    const SafeAccountInfo *account_infos,
    int account_infos_len
) {
  const SafeSignerSeeds signers_seeds[] = {{}};
  return sol_invoke_signed(
    instruction,
    account_infos,
    account_infos_len,
    signers_seeds,
    0
  );
}

#ifdef __cplusplus
}
#endif

/**@}*/
