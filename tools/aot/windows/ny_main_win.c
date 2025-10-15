#include <stdint.h>
#include <stdio.h>

// Declare external symbols with asm aliases for dotted exports
extern int64_t nyash_box_from_i8_string(const char*) asm("nyash.box.from_i8_string");
extern int64_t nyash_string_concat_hh(int64_t, int64_t) asm("nyash.string.concat_hh");
extern int64_t nyash_string_len_h(int64_t) asm("nyash.string.len_h");

// Main entry point
__declspec(dllexport) int64_t ny_main(void) {
    // Create string boxes
    const char* str1 = "Main(";
    const char* str2 = ")";

    int64_t h1 = nyash_box_from_i8_string(str1);
    int64_t h2 = nyash_box_from_i8_string(str2);

    // Concatenate strings
    int64_t result = nyash_string_concat_hh(h1, h2);

    // Get length (optional test)
    int64_t len = nyash_string_len_h(result);

    printf("Result: %lld\n", (long long)len);
    return len;
}

// C main wrapper
int main(void) {
    int64_t result = ny_main();
    return (int)(result & 0xFF);
}
