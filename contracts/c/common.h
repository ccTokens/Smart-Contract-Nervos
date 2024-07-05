#ifndef XUDT_EXTENSION_COMMON_H
#define XUDT_EXTENSION_COMMON_H

#include <limits.h>
#include "entry.h"
#include "ckb_syscalls.h"
#include "structures.h"
#include "ckb_consts.h"

#define MAX_WITNESS_SIZE 32768
#define TMP_SIZE 32768
#define TMP_SIZE_SHORT 1024
#define MAX_CELL_CAPACITY 1024
#define MAX_CELL_NUM 256
#define BLAKE2B_BLOCK_SIZE 32
#define BLAKE160_SIZE 20
#define TOKEN_ID_LEN 20
#define U128_BYTES 16
#define HASH_SIZE 32
#define TYPE_ID_SIZE 32
#define MEMBER_ID_SIZE 20
#define TICK_CELL_CURRENT_VERSION 0
#define GOVERNANCE_CELL_CURRENT_VERSION 0
#define DEBUG_BUFFER_SIZE (64 * 1024)

#ifdef CKB_C_STDLIB_PRINTF
#define debug_print(s) ckb_debug(s)
#define debug_print_int(prefix, value) debug_print_int_impl((prefix), (value))
#define debug_print_data(prefix, data, data_len) debug_print_data_impl((prefix), (data), (data_len))
#define debug_print_string(prefix, data, data_len) debug_print_string_impl((prefix), (data), (data_len))

static char debug_buffer[DEBUG_BUFFER_SIZE];
static void debug_print_data_impl(const char *prefix,
                                  const uint8_t *data,
                                  uint32_t data_len)
{
    int offset = 0;
    int space_left = DEBUG_BUFFER_SIZE - 1; // Leave space for null terminator

    // Write the prefix
    int prefix_len = snprintf_(debug_buffer, space_left, "%s", prefix);
    if (prefix_len >= space_left)
    {
        // Not enough space for prefix, log error or handle appropriately
        ckb_debug("Prefix too long for buffer");
        return;
    }
    offset += prefix_len;
    space_left -= prefix_len;

    // Write the data
    for (size_t i = 0; i < data_len; i++)
    {
        int bytes_written = snprintf_(debug_buffer + offset, space_left, "%02x", data[i]);
        if (bytes_written >= space_left)
        {
            // Not enough space for more data, log warning or handle appropriately
            ckb_debug("Buffer full, truncating data");
            break;
        }
        offset += bytes_written;
        space_left -= bytes_written;
    }

    debug_buffer[offset] = '\0';
    ckb_debug(debug_buffer);
}

static void debug_print_int_impl(const char *prefix, int ret)
{
    int offset = 0;
    offset += sprintf_(debug_buffer, "%s(%d)", prefix, ret);
    debug_buffer[offset] = '\0';
    ckb_debug(debug_buffer);
}
static void debug_print_string_impl(const char *prefix,
                                    const uint8_t *data,
                                    uint32_t data_len)
{
    int offset = 0;
    offset += sprintf_(debug_buffer, "%s", prefix);
    for (size_t i = 0; i < data_len; i++)
    {
        offset += sprintf_(debug_buffer + offset, "%c", data[i]);
    }
    debug_buffer[offset] = '\0';
    ckb_debug(debug_buffer);
}
#else
#define debug_print(s)
#define debug_print_int(prefix, value)
#define debug_print_data(prefix, data, data_len)
#define debug_print_string(prefix, data, data_len)
#endif

#define CHECK(_code)        \
    do                      \
    {                       \
        int code = (_code); \
        if (code != 0)      \
        {                   \
            err = code;     \
            goto exit;      \
        }                   \
    } while (0)

#define CHECK2(cond, code) \
    do                     \
    {                      \
        if (!(cond))       \
        {                  \
            err = code;    \
            goto exit;     \
        }                  \
    } while (0)

/* calculate inputs length */
int calculate_inputs_len()
{
    uint64_t len = 0;
    /* lower bound, at least tx has one input */
    int lo = 0;
    /* higher bound */
    int hi = 4;
    int ret;
    /* try to load input until failing to increase lo and hi */
    while (1)
    {
        ret = ckb_load_input_by_field(NULL, &len, 0, hi, CKB_SOURCE_INPUT,
                                      CKB_INPUT_FIELD_SINCE);
        if (ret == CKB_SUCCESS)
        {
            lo = hi;
            // Check for potential overflow before doubling hi
            if (hi > INT_MAX / 2)
            {
                // Handle the overflow case appropriately, e.g., set hi to a maximum safe value
                hi = INT_MAX;
            }
            else
            {
                hi *= 2;
            }
        }
        else
        {
            break;
        }
    }

    /* now we get our lower bound and higher bound,
     count number of inputs by binary search */
    int i;
    while (lo + 1 != hi)
    {
        i = (lo + hi) / 2;
        ret = ckb_load_input_by_field(NULL, &len, 0, i, CKB_SOURCE_INPUT,
                                      CKB_INPUT_FIELD_SINCE);
        if (ret == CKB_SUCCESS)
        {
            lo = i;
        }
        else
        {
            hi = i;
        }
    }
    /* now lo is last input index and hi is length of inputs */
    return hi;
}

// Function to compare the array with the provided string
bool matches_string(const char *str, const uint8_t *action, size_t size)
{
    // Check if the length of the string matches the size of the array
    if (size != strlen(str))
    {
        return false;
    }

    // Compare each character's ASCII value to the integer in the array
    for (int i = 0; i < size; i++)
    {
        if (action[i] != str[i])
        {
            return false; // Return false if any character does not match
        }
    }

    return true; // All elements matched
}
#endif // XUDT_EXTENSION_COMMON_H
