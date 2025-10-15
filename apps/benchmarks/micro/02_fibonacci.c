// Fibonacci(12) - Recursive version
// C言語版 - Hakoruneと同じロジック

#include <stdio.h>

int fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main(void) {
    int result = fib(12);
    printf("Result: %d\n", result);
    return 0;
}
