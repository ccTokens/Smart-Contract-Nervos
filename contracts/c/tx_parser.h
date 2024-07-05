#ifndef XUDT_DEMO_TX_PARSER_H
#define XUDT_DEMO_TX_PARSER_H

#include "common.h"
#include "basic.h"
#include "cell.h"
#include "utils.h"

int find_cells_by_code_hash(size_t *cell_indices, size_t *cell_indices_len, int field, int source, uint8_t *expected_hash)
{
    int err = 0;
    size_t i = 0, j = 0;

    debug_print_data("try to find cells by code hash: ", expected_hash, BLAKE2B_BLOCK_SIZE);
    debug_print_int("search source: ", source);
    debug_print_int("search field: ", field);
    uint8_t buff[TMP_SIZE];
    uint64_t len = TMP_SIZE;
    for (i = 0; i < MAX_CELL_NUM; i++)
    {
        len = TMP_SIZE;
        // debug_print_int("try to load cell by field: idx", (int)i);
        err = ckb_checked_load_cell_by_field(buff, &len, 0, i, source, field);
        // debug_print_int("load cell by field: ret", err);
        // debug_print_data("loaded cell[0..64]: ", buff, 64);
        if (err == CKB_INDEX_OUT_OF_BOUND)
        {
            break;
        }
        if (err == CKB_ITEM_MISSING)
        { // just no type script
            continue;
        }
        if (err != 0)
        {
            return err;
        }

        if (memcmp(expected_hash, buff + 16, BLAKE2B_BLOCK_SIZE) == 0)
        {
            debug_print_int("cell found, index", i);
            // check if it's a type script
            if (buff[16 + BLAKE2B_BLOCK_SIZE] != 1)
            {
                break;
            }
            cell_indices[j] = i;
            j++;
            // avoid overflow
            if (j == *cell_indices_len)
            {
                break;
            }
        }
    }
    *cell_indices_len = j;
    return 0;
}

uint128_t get_sudt_amount(size_t cell_index, int source)
{
    uint128_t current_amount = 0;
    uint64_t len = 16;

    int ret = ckb_load_cell_data((uint8_t *)&current_amount, &len, 0,
                                 cell_index, source);
    if (ret != CKB_SUCCESS)
    {
        return ret;
    }

    return current_amount;
}

// copied from simple_udt.c and modified
int simple_udt(uint128_t *ia, uint128_t *oa)
{
    int ret = 0;
    uint128_t input_amount = 0;
    size_t i = 0;
    uint64_t len = 0;
    while (1)
    {
        uint128_t current_amount = 0;
        len = 16;
        ret = ckb_load_cell_data((uint8_t *)&current_amount, &len, 0, i, CKB_SOURCE_GROUP_INPUT);
        if (ret == CKB_INDEX_OUT_OF_BOUND)
        {
            break;
        }
        debug_print_data("input current_amount: ", (uint8_t *)&current_amount, 16);
        if (ret != CKB_SUCCESS)
        {
            return ret;
        }
        if (len < 16)
        {
            return ERROR_ENCODING;
        }
        input_amount += current_amount;
        // Like any serious smart contract out there, we will need to check for overflows.
        if (input_amount < current_amount)
        {
            return ERROR_OVERFLOWING;
        }
        i += 1;
    }

    uint128_t output_amount = 0;
    i = 0;
    while (1)
    {
        uint128_t current_amount = 0;
        len = 16;
        ret = ckb_load_cell_data((uint8_t *)&current_amount, &len, 0, i,
                                 CKB_SOURCE_GROUP_OUTPUT);
        if (ret == CKB_INDEX_OUT_OF_BOUND)
        {
            break;
        }
        debug_print_data("output current_amount: ", (uint8_t *)&current_amount, 16);

        if (ret != CKB_SUCCESS)
        {
            return ret;
        }
        if (len < 16)
        {
            return ERROR_ENCODING;
        }
        output_amount += current_amount;
        if (output_amount < current_amount)
        {
            return ERROR_OVERFLOWING;
        }
        i += 1;
    }

    debug_print_int("input total amount: ", input_amount);
    debug_print_int("output total amount: ", output_amount);

    *ia = input_amount;
    *oa = output_amount;
    return CKB_SUCCESS;
}

Action get_tx_action()
{
    int err = 0;
    uint8_t action_data[TMP_SIZE_SHORT];
    uint64_t action_data_len = TMP_SIZE_SHORT;

    int action_witness_index = calculate_inputs_len();
    debug_print_int("action_witness_index", action_witness_index);
    err = ckb_load_witness(action_data, &action_data_len, 0, action_witness_index, CKB_SOURCE_INPUT);
    debug_print_int("load action data, ret", err);

    // if error, maybe don't have action, it's normal transfer
    if (err != 0)
    {
        return Others;
    }

    // if action data is empty or too long, it's not a valid action
    if (action_data_len == 0 || action_data_len > TMP_SIZE_SHORT)
    {
        return Others;
    }

    // jump up the first byte, it's the version
    uint8_t *action = action_data + 1;
    int action_len = action_data_len - 1;
    debug_print_string("action_data: ", action, action_len);

    for (int i = 0; i < NUM_ACTIONS; i++)
    {
        if (memcmp(ACTIONS[i], action, action_len) == 0)
        {
            return (Action)i;
        }
    }
    return Others;
}

int get_type_id(uint8_t *type_id, ContractType type_)
{
    // get config cell type id
    uint8_t config_cell_type_id[HASH_SIZE];
    hex2str(CONFIG_CELL_TYPE_ID, config_cell_type_id);
    debug_print_data("config_cell_type_id: ", config_cell_type_id, HASH_SIZE);
    // get config cell idx
    size_t config_cells[MAX_CELL_NUM];
    size_t config_cells_len = MAX_CELL_NUM;
    int err = find_cells_by_code_hash(config_cells, &config_cells_len, CKB_CELL_FIELD_TYPE, CKB_SOURCE_CELL_DEP, config_cell_type_id);
    debug_print_int("find config cells ret", err);
    if (err != 0)
    {
        return err;
    }
    if (config_cells_len != 1)
    {
        debug_print_int("config_cells_len error，expect 1, actual", config_cells_len);
        return ERROR_CONFIG_CELL_NUM;
    }
    // load config cell data
    uint8_t config_data[TMP_SIZE];
    uint64_t config_data_len = TMP_SIZE;
    err = ckb_checked_load_cell_data(config_data, &config_data_len, 0, config_cells[0], CKB_SOURCE_CELL_DEP);
    debug_print_int("load config cell data, ret", err);
    if (err != 0)
    {
        return err;
    }
    // parse it
    mol_seg_t config_seg;
    config_seg.ptr = config_data + 1; //+1 to skip version
    config_seg.size = config_data_len - 1;
    // debug_print_data("config_data: ", config_data, config_data_len);

    mol_errno mol_err = MolReader_BytesVec_verify(&config_seg, false);
    debug_print_int("MolReader_BytesVec_verify, ret", mol_err);
    if (mol_err != MOL_OK)
    {
        return ERROR_ENCODING;
    }
    mol_seg_res_t config_seg_res = MolReader_BytesVec_get(&config_seg, type_);
    if (config_seg_res.errno != MOL_OK)
    {
        debug_print_int("verify molecule BytesVec error", config_seg_res.errno);
        return ERROR_ENCODING;
    }
    mol_seg_t config = config_seg_res.seg;
    // debug_print_data("config_bytes_seg.ptr: ", config.ptr, config.size);
    uint32_t header;
    memcpy(&header, config.ptr + 4, 4); // 4 is the length of molecule header
    if (header != type_)
    {
        debug_print_int("header error, expect", type_);
        debug_print_int("header error, actual", header);
        return ERROR_ENCODING;
    }
    memcpy(type_id, config.ptr + 8, TYPE_ID_SIZE); // 8 is the offset of type_id

    return 0;
}
#endif // XUDT_DEMO_TX_PARSER_H
