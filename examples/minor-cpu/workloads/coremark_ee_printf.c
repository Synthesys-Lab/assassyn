#include "coremark.h"

#include <stdarg.h>

volatile ee_u32 coremark_status = 0;

enum
{
    COREMARK_STATUS_OK = 0x7fffffff,
    COREMARK_STATUS_GENERIC_ERROR = 1,
    COREMARK_STATUS_LIST_ERROR = 2,
    COREMARK_STATUS_MATRIX_ERROR = 3,
    COREMARK_STATUS_STATE_ERROR = 4,
    COREMARK_STATUS_UNVALIDATED = 5,
};

static int starts_with(const char *text, const char *prefix)
{
    while (*prefix != '\0')
    {
        if (*text != *prefix)
            return 0;
        text++;
        prefix++;
    }
    return 1;
}

int ee_printf(const char *fmt, ...)
{
    va_list args;

    va_start(args, fmt);
    va_end(args);

    if (starts_with(fmt, "Correct operation validated"))
        coremark_status = COREMARK_STATUS_OK;
    else if (starts_with(fmt, "[%u]ERROR! list crc"))
        coremark_status = COREMARK_STATUS_LIST_ERROR;
    else if (starts_with(fmt, "[%u]ERROR! matrix crc"))
        coremark_status = COREMARK_STATUS_MATRIX_ERROR;
    else if (starts_with(fmt, "[%u]ERROR! state crc"))
        coremark_status = COREMARK_STATUS_STATE_ERROR;
    else if (starts_with(fmt, "Errors detected"))
    {
        if (coremark_status == 0)
            coremark_status = COREMARK_STATUS_GENERIC_ERROR;
    }
    else if (starts_with(fmt, "Cannot validate operation"))
        coremark_status = COREMARK_STATUS_UNVALIDATED;
    else if (starts_with(fmt, "ERROR"))
        coremark_status = COREMARK_STATUS_GENERIC_ERROR;

    return 0;
}
