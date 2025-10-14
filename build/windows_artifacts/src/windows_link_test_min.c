#include <stdint.h>
#include <stdio.h>

// Bind to dotted export names via asm aliases
extern int64_t nyash_box_from_i8_string(const char*) asm("nyash.box.from_i8_string");
extern int64_t nyash_string_concat_hh(int64_t, int64_t) asm("nyash.string.concat_hh");
extern int64_t nyash_string_len_h(int64_t) asm("nyash.string.len_h");

int main(void) {
    int64_t a = nyash_box_from_i8_string("hi");
    int64_t b = nyash_box_from_i8_string("yo");
    int64_t c = nyash_string_concat_hh(a, b);
    int64_t len = nyash_string_len_h(c);
    if (len == 4) {
        printf("Result: %lld\n", (long long)len);
        return 0;
    } else {
        printf("Unexpected len=%lld\n", (long long)len);
        return (int)len;
    }
}
