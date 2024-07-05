#include "common.h"
#include "tx_parser.h"
#include "utils.h"
#include "xudt_rce.h"
#include "basic.h"
#include "cell.h"

__attribute__((visibility("default"))) int
validate(
    int is_owner_mode,      // to indicate if the script is in owner mode
    size_t extension_index, // to indicate the index of the current extension in ScriptVec structure.
    const unsigned char *args,
    size_t args_length)
{
    int err = 0;

    debug_print("<<<<<<<<<<<< Enter dyn lib, xudt extension");
    debug_print_int("is_owner_mode", is_owner_mode);
    debug_print_int("extension_index", extension_index);
    debug_print_data("args: ", args, args_length); //
    debug_print_int("args_length", args_length);

    uint128_t input_sudt_amount, output_sudt_amount;
    simple_udt(&input_sudt_amount, &output_sudt_amount);
    debug_print_int("input_sudt_amount", (int)input_sudt_amount);
    debug_print_int("output_sudt_amount", (int)output_sudt_amount);

    // Check only in non-transfer scenarios, including minting or burning.
    // All other checks should take place in the tick-cell-type contract.
    debug_print("Check whether the token amount is the same.");
    if (input_sudt_amount != output_sudt_amount)
    {
        // Only owner can mint or burn
        debug_print("Check only owner can mint or burn.");
        CHECK2(is_owner_mode == 1, ERROR_OWNER_MODE);
    }

    // The args is token_id, we'll check its length.
    CHECK2(args_length == TOKEN_ID_LEN, ERROR_TOKEN_ID_SIZE);

    return 0;
exit:
    return err;
}