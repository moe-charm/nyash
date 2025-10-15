/* The Computer Language Benchmarks Game
   Binary Trees benchmark - 5 second fixed-time version
   Based on original by Kevin Carson
*/

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <sys/time.h>

typedef struct tn {
    struct tn*    left;
    struct tn*    right;
    long          item;
} treeNode;

treeNode* NewTreeNode(treeNode* left, treeNode* right, long item)
{
    treeNode* new = (treeNode*)malloc(sizeof(treeNode));
    new->left = left;
    new->right = right;
    new->item = item;
    return new;
}

long ItemCheck(treeNode* tree)
{
    if (tree->left == NULL)
        return tree->item;
    else
        return tree->item + ItemCheck(tree->left) - ItemCheck(tree->right);
}

treeNode* BottomUpTree(long item, unsigned depth)
{
    if (depth > 0)
        return NewTreeNode(
            BottomUpTree(2 * item - 1, depth - 1),
            BottomUpTree(2 * item, depth - 1),
            item
        );
    else
        return NewTreeNode(NULL, NULL, item);
}

void DeleteTree(treeNode* tree)
{
    if (tree->left != NULL) {
        DeleteTree(tree->left);
        DeleteTree(tree->right);
    }
    free(tree);
}

long long current_time_ms() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (long long)tv.tv_sec * 1000 + tv.tv_usec / 1000;
}

int main(int argc, char* argv[])
{
    unsigned N = (argc > 1) ? atol(argv[1]) : 10;
    unsigned minDepth = 4;
    unsigned maxDepth = (minDepth + 2 > N) ? minDepth + 2 : N;
    unsigned stretchDepth = maxDepth + 1;

    // Fixed 5-second benchmark
    long long start_time = current_time_ms();
    long iterations = 0;
    long final_check = 0;

    while (current_time_ms() - start_time < 5000) {
        // Stretch tree
        treeNode* stretchTree = BottomUpTree(0, stretchDepth);
        long check = ItemCheck(stretchTree);
        DeleteTree(stretchTree);

        // Long-lived tree
        treeNode* longLivedTree = BottomUpTree(0, maxDepth);

        // Create and check many trees
        for (unsigned depth = minDepth; depth <= maxDepth; depth += 2) {
            long num_iterations = 1L << (maxDepth - depth + minDepth);
            long check_sum = 0;

            for (long i = 1; i <= num_iterations; i++) {
                treeNode* tempTree = BottomUpTree(i, depth);
                check_sum += ItemCheck(tempTree);
                DeleteTree(tempTree);

                tempTree = BottomUpTree(-i, depth);
                check_sum += ItemCheck(tempTree);
                DeleteTree(tempTree);
            }
        }

        final_check = ItemCheck(longLivedTree);
        DeleteTree(longLivedTree);
        iterations++;
    }

    long long elapsed_ms = current_time_ms() - start_time;
    long ops_per_sec = (iterations * 1000) / elapsed_ms;
    long us_per_op = (elapsed_ms * 1000) / iterations;

    printf("Binary Trees (depth %u) - 5 seconds\n", N);
    printf("\n");
    printf("Iterations: %ld\n", iterations);
    printf("Elapsed: %lld ms\n", elapsed_ms);
    printf("Ops/sec: %ld\n", ops_per_sec);
    printf("µs/op: %ld\n", us_per_op);
    printf("Last result: %ld\n", final_check);

    return 0;
}
