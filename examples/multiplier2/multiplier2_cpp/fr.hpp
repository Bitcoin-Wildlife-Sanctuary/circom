#ifndef __FR_H
#define __FR_H

#include <stdint.h>

#define Fr_N64 1
typedef uint64_t FrRawElement[1];
typedef struct {
    FrRawElement longVal;
} FrElement;
typedef FrElement *PFrElement;

const FrElement Fr_q = FrElement {
        { 2147483647 }
};

void Fr_copy(PFrElement r, PFrElement a);
void Fr_copyn(PFrElement r, PFrElement a, int n);

void Fr_add(PFrElement r, PFrElement a, PFrElement b);
void Fr_sub(PFrElement r, PFrElement a, PFrElement b);
void Fr_neg(PFrElement r, PFrElement a);
void Fr_mul(PFrElement r, PFrElement a, PFrElement b);
void Fr_band(PFrElement r, PFrElement a, PFrElement b);
void Fr_bor(PFrElement r, PFrElement a, PFrElement b);
void Fr_bxor(PFrElement r, PFrElement a, PFrElement b);
void Fr_bnot(PFrElement r, PFrElement a);
void Fr_shl(PFrElement r, PFrElement a, PFrElement b);
void Fr_shr(PFrElement r, PFrElement a, PFrElement b);
void Fr_eq(PFrElement r, PFrElement a, PFrElement b);
void Fr_neq(PFrElement r, PFrElement a, PFrElement b);
void Fr_lt(PFrElement r, PFrElement a, PFrElement b);
void Fr_gt(PFrElement r, PFrElement a, PFrElement b);
void Fr_leq(PFrElement r, PFrElement a, PFrElement b);
void Fr_geq(PFrElement r, PFrElement a, PFrElement b);
void Fr_land(PFrElement r, PFrElement a, PFrElement b);
void Fr_lor(PFrElement r, PFrElement a, PFrElement b);
void Fr_lnot(PFrElement r, PFrElement a);
void Fr_toLongNormal(PFrElement r, PFrElement a);

int Fr_isTrue(PFrElement pE);
int Fr_toInt(PFrElement pE);

void Fr_str2element(PFrElement pE, char const *s, unsigned int base);
char *Fr_element2str(PFrElement pE);
void Fr_idiv(PFrElement r, PFrElement a, PFrElement b);
void Fr_mod(PFrElement r, PFrElement a, PFrElement b);
void Fr_div(PFrElement r, PFrElement a, PFrElement b);
void Fr_pow(PFrElement r, PFrElement a, PFrElement b);

#endif // __FR_H



