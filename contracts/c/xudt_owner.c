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
    debug_print("<<<<<<<<<<<< Enter dyn lib, xudt owner");
    debug_print_int("is_owner_mode", is_owner_mode);
    debug_print_int("extension_index", extension_index);
    debug_print_data("args: ", args, args_length);
    debug_print_int("args_length", args_length);

    // init temp buffer
    uint8_t tmp[TMP_SIZE];
    size_t tmp_len = TMP_SIZE;

    // get all governance cells
    size_t governance_cells[MAX_CELL_NUM];
    size_t governance_cells_len = MAX_CELL_NUM;
    uint8_t governance_cell_type_id[HASH_SIZE];
    get_type_id(governance_cell_type_id, GovernanceMemberCellTypeId);
    debug_print_data("governance_cell_type_id: ", governance_cell_type_id, HASH_SIZE);
    int err = find_cells_by_code_hash(governance_cells, &governance_cells_len, CKB_CELL_FIELD_TYPE,
                                      CKB_SOURCE_CELL_DEP, governance_cell_type_id);
    debug_print_int("find governance cells ret", err);

    CHECK(err);
    debug_print_int("governance_cells_len", governance_cells_len);
    CHECK2(governance_cells_len == 1, ERROR_GOVERNANCE_CELL_NUM);

    size_t omni_lock_cells[MAX_CELL_NUM];
    size_t omni_lock_cells_len = MAX_CELL_NUM;
    uint8_t omni_lock_cell_type_id[TYPE_ID_SIZE];
    get_type_id(omni_lock_cell_type_id, OmniLockTypeId);
    debug_print_data("omni_lock_cell_type_id: ", omni_lock_cell_type_id, HASH_SIZE);
    err = find_cells_by_code_hash(omni_lock_cells, &omni_lock_cells_len, CKB_CELL_FIELD_LOCK,
                                  CKB_SOURCE_INPUT, omni_lock_cell_type_id);
    debug_print_int("find omni lock cells ret", err);
    CHECK(err);
    debug_print_int("omni_lock_cells_len", omni_lock_cells_len);
    CHECK2(omni_lock_cells_len > 0, ERROR_OMNI_LOCK_CELL_NUM);

    // try to get governance data
    uint8_t governance_data[TMP_SIZE];
    uint64_t governance_data_len = TMP_SIZE;
    err = ckb_checked_load_cell_data(governance_data, &governance_data_len, 0, governance_cells[0], CKB_SOURCE_CELL_DEP);
    debug_print_int("load governance cell data, ret", err);
    CHECK(err);
    debug_print_data("governance_data: ", governance_data, governance_data_len);

    int version = governance_data[0];
    if (version < 0 || version > GOVERNANCE_CELL_CURRENT_VERSION)
    {
        debug_print_int("verify governance version error, version", version);
        return ERROR_GOVERNANCE_VERSION;
    }
    mol_seg_t governance_seg;
    governance_seg.ptr = governance_data + 1;
    governance_seg.size = governance_data_len - 1;

    mol_errno mol_err = MolReader_GovernanceMembers_verify(&governance_seg, false);
    debug_print_int("get governance members, ret", mol_err);
    CHECK2(mol_err == MOL_OK, ERROR_ENCODING);

    mol_seg_t governance_members_lock_args_seg = MolReader_GovernanceMembers_get_lock_args(&governance_seg);
    debug_print_data("governance_members_lock_args_seg.ptr: ", governance_members_lock_args_seg.ptr,
                     governance_members_lock_args_seg.size);

    // get all omni lock cells and their lock args
    for (int i = 0; i < omni_lock_cells_len; i++)
    {
        size_t omni_lock_index = omni_lock_cells[i];
        debug_print_int("omni_lock_cell_index", omni_lock_index);

        tmp_len = TMP_SIZE;
        err = ckb_checked_load_cell_by_field(tmp, &tmp_len, 0, omni_lock_index, CKB_SOURCE_INPUT,
                                             CKB_CELL_FIELD_LOCK);
        debug_print_int("load omni lock cell lock, ret", err);
        CHECK(err);

        mol_seg_t omni_lock_seg;
        omni_lock_seg.ptr = tmp;
        omni_lock_seg.size = tmp_len;
        mol_err = MolReader_Script_verify(&omni_lock_seg, false);
        CHECK2(mol_err == MOL_OK, ERROR_ENCODING);

        mol_seg_t omni_lock_args_seg = MolReader_Script_get_args(&omni_lock_seg);
        debug_print_data("omni_lock_args_seg.ptr: ", omni_lock_args_seg.ptr, omni_lock_args_seg.size);

        // Check whether any Omni lock cell argument is equal to the Governance member's lock argument.
        if (omni_lock_args_seg.size == governance_members_lock_args_seg.size &&
            memcmp(omni_lock_args_seg.ptr, governance_members_lock_args_seg.ptr, omni_lock_args_seg.size) == 0)
        {
            debug_print_int("Found one Omni lock cell args is equal to the Governance member's lock args at index", i);
            return 0;
        }
    }

    // No Omni lock cell argument is equal to the Governance member's lock argument.
    debug_print("No Omni lock cell argument is equal to the Governance member's lock argument.");
    return ERROR_UNAUTHORIZED_GOVERNANCE_MEMBER;

exit:
    return err;
}