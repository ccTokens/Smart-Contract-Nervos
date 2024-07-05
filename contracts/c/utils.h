#ifndef XUDT_EXTENSION_UTILS_H
#define XUDT_EXTENSION_UTILS_H
#include "common.h"

char char2hex(char hexChar) {
    char tmp;

    if (hexChar >= '0' && hexChar <= '9') {
        tmp = hexChar - '0';
    } else if (hexChar >= 'A' && hexChar <= 'F') {
        tmp = hexChar - 'A' + 10;
    } else if (hexChar >= 'a' && hexChar <= 'f') {
        tmp = hexChar - 'a' + 10;
    } else {
        // illegal character
        tmp = -1;
    }

    return tmp;
}

int hex2str(const char* _hexStr, unsigned char* _str) {
    int i;
    int len;
    unsigned char* ptr;
    if(NULL == _str || NULL == _hexStr)
    {
        return -1;
    }

    len = strlen(_hexStr);
    ptr = _str;

    for(i=0; i<len-1;i++) {
        *ptr = char2hex(_hexStr[i])*16;
        i++;
        *ptr += char2hex(_hexStr[i]);
        ptr++;
    }
    return 0;
}
#endif //XUDT_EXTENSION_UTILS_H
