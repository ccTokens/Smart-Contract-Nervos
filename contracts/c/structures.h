#ifndef XUDT_EXTENSION_STRUCTURES_H
#define XUDT_EXTENSION_STRUCTURES_H

#ifdef CKB_TESTNET // testnet
// testnet env
const char *CONFIG_CELL_TYPE_ID = "1fa21d5beb92fdf044f27f6310564be88f59e32557abf44d0db30bc239e14ff3";
// dev env
// const char *CONFIG_CELL_TYPE_ID = "c25886ca81aafbe4c92fa3243e7973557dd762ea3ac6ce5b0cca363ad8d2e53b";

#else // mainnet
const char *CONFIG_CELL_TYPE_ID = "470452746a7abdb1f1723c5bd10d8b5bcda0dc4f00881cb9c6d8cf84b697d475";
#endif

typedef unsigned __int128 uint128_t;

enum ErrorCode
{
    // 0 is the only success code. We can use 0 directly.

    // inherit from simple_udt
    ERROR_ARGUMENTS_LEN = -1,
    ERROR_ENCODING = -2,
    ERROR_SYSCALL = -3,
    ERROR_SCRIPT_TOO_LONG = -21,
    ERROR_OVERFLOWING = -51,
    ERROR_AMOUNT = -52,

    // error code is starting from 70, to avoid conflict with
    // common error code in other scripts.
    ERROR_TICK_TYPE = 70,
    ERROR_TICK_VERSION = 71,
    ERROR_TICK_DATA = 72,
    ERROR_TICK_VALUE = 73,
    ERROR_TICK_CELL_NUM = 74,

    ERROR_OWNER_MODE = 80,
    ERROR_TOKEN_ID_NOT_MATCH = 81,
    ERROR_TOKEN_ID_SIZE = 82,
    ERROR_TOKEN_ID = 83,

    ERROR_XUDT_AMOUNT = 90,
    ERROR_XUDT_OWNER_MODE,

    ERROR_GOVERNANCE_CELL_NUM,
    ERROR_GOVERNANCE_VERSION,
    ERROR_UNAUTHORIZED_GOVERNANCE_MEMBER,
    ERROR_OMNI_LOCK_CELL_NUM,
    ERROR_NOT_SUPPORTED,
    ERROR_CONFIG_CELL_NUM,

};

typedef enum OperationType
{
    Mint = 0,
    Burn = 1,
    Transfer = 2,
} OperationType;

typedef enum Action
{
    DeployConfig,
    UpdateConfig,
    InitGovernance,
    UpdateOwner,
    UpdateCustodians,
    UpdateMerchants,
    DeployToken,
    RequestMint,
    ConfirmMint,
    RejectMint,
    RequestBurn,
    ConfirmBurn,
    RejectBurn,
    Others,
} Action;

const char *ACTIONS[] = {
    "deploy_config",
    "update_config",
    "init_governance",
    "update_owner",
    "update_custodians",
    "update_merchants",
    "deploy_token",
    "request_mint",
    "confirm_mint",
    "reject_mint",
    "request_burn",
    "confirm_burn",
    "reject_burn",
    "Others"};

#define NUM_ACTIONS 14 // Number of elements in the array
/*
 * 0
System status
1
governance-member-cell-type-id
2
governance-member-type-args
3
tick-cell-type-id
4
tick-type-args
5
xudt-info-cell-type-id
6
xudt-info-type-args
7
xudt-cell-type-id
8
xudt-type-args
9
always-success-type-id
10
always-success-type-args
11
omni-lock-type-id
12
omni-lock-type-args
13
xudt-extenstion-type-id
14
xudt-extenstion-type-args
15
xudt-owner-type-id
16
xudt-owner-type-args
 */
typedef enum ContractType
{
    SystemStatus = 0,
    GovernanceMemberCellTypeId = 1,
    GovernanceMemberTypeArgs = 2,
    TickCellTypeId = 3,
    TickTypeArgs = 4,
    XudtInfoCellTypeId = 5,
    XudtInfoTypeArgs = 6,
    XudtCellTypeId = 7,
    XudtTypeArgs = 8,
    AlwaysSuccessTypeId = 9,
    AlwaysSuccessTypeArgs = 10,
    OmniLockTypeId = 11,
    OmniLockTypeArgs = 12,
    XudtExtensionTypeId = 13,
    XudtExtensionTypeArgs = 14,
    XudtOwnerTypeId = 15,
    XudtOwnerTypeArgs = 16,
} ContractType;

#endif // XUDT_EXTENSION_STRUCTURES_H
