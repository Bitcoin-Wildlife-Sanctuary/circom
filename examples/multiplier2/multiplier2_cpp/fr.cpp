#include "fr.hpp"
#include <stdio.h>
#include <stdlib.h>
#include <gmp.h>

const uint64_t MOD = 2147483647;

void Fr_copy(PFrElement r, PFrElement a) {
    r->longVal[0] = a->longVal[0];
}

void Fr_copyn(PFrElement r, PFrElement a, int n) {
    for(int i  = 0; i < n; i++) {
        (r++)->longVal[0] = (a++)->longVal[0];
    }
}

void Fr_add(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (a->longVal[0] + b->longVal[0]) % MOD;
}

void Fr_sub(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (MOD + a->longVal[0] - b->longVal[0]) % MOD;
}

void Fr_neg(PFrElement r, PFrElement a) {
    r->longVal[0] = (MOD - a->longVal[0]) % MOD;
}

void Fr_mul(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (uint32_t) (a->longVal[0] * b->longVal[0] % MOD);
}

void Fr_band(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (a->longVal[0] & b->longVal[0]) % MOD;
}

void Fr_bor(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (a->longVal[0] | b->longVal[0]) % MOD;
}

void Fr_bxor(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = (a->longVal[0] ^ b->longVal[0]) % MOD;
}

void Fr_bnot(PFrElement r, PFrElement a) {
    r->longVal[0] = (~a->longVal[0]) % MOD;
}

void Fr_shl(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = a->longVal[0];
    for(int i = 0; i < b->longVal[0]; i++) {
        r->longVal[0] = (r->longVal[0] << 1) % MOD;
    }
}

void Fr_shr(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = a->longVal[0];
    for(int i = 0; i < b->longVal[0]; i++) {
        r->longVal[0] = r->longVal[0] >> 1;
    }
}

void Fr_eq(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] == b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_neq(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] != b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_lt(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] < b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_gt(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] > b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_leq(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] <= b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_geq(PFrElement r, PFrElement a, PFrElement b) {
    if(a->longVal[0] >= b->longVal[0]) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_land(PFrElement r, PFrElement a, PFrElement b) {
    if((a->longVal[0] != 0) && (b->longVal[0] != 0)) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_lor(PFrElement r, PFrElement a, PFrElement b) {
    if((a->longVal[0] != 0) || (b->longVal[0] != 0)) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_lnot(PFrElement r, PFrElement a) {
    if(a->longVal[0] != 0) {
        r->longVal[0] = 1;
    } else {
        r->longVal[0] = 0;
    }
}

void Fr_toLongNormal(PFrElement r, PFrElement a) {
    r->longVal[0] = a->longVal[0];
}

int Fr_isTrue(PFrElement pE) {
    if(pE->longVal[0] != 0) {
        return 1;
    } else {
        return 0;
    }
}

int Fr_toInt(PFrElement pE) {
    return pE->longVal[0];
}

void Fr_fromMpz(PFrElement pE, mpz_t v) {
    pE->longVal[0] = mpz_get_si(v) % MOD;
}

void Fr_str2element(PFrElement pE, char const *s, unsigned int base) {
    mpz_t mr;
    mpz_t mq;
    mpz_init_set_str(mr, s, base);
    mpz_init(mq);
    mpz_set_si(mq, MOD);
    mpz_fdiv_r(mr, mr, mq);
    Fr_fromMpz(pE, mr);
    mpz_clear(mr);
    mpz_clear(mq);
}

char *Fr_element2str(PFrElement pE) {
    char *r = new char[32];
    snprintf(r, 32, "%d", (uint32_t) pE->longVal[0]);
    return r;
}

void Fr_idiv(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = a->longVal[0] / b->longVal[0];
}

void Fr_mod(PFrElement r, PFrElement a, PFrElement b) {
    r->longVal[0] = a->longVal[0] % b->longVal[0];
}

void Fr_toMpz(mpz_t r, PFrElement pE) {
    mpz_set_si(r, pE->longVal[0]);
}

void Fr_inv(PFrElement r, PFrElement a) {
    mpz_t ma;
    mpz_t mr;
    mpz_t mq;
    mpz_init(ma);
    mpz_init(mr);
    mpz_init(mq);

    Fr_toMpz(ma, a);
    mpz_set_si(mq, MOD);
    mpz_invert(mr, ma, mq);
    Fr_fromMpz(r, mr);
    mpz_clear(ma);
    mpz_clear(mr);
    mpz_clear(mq);
}

void Fr_div(PFrElement r, PFrElement a, PFrElement b) {
    Fr_inv(r, b);
    Fr_mul(r, r, a);
}

void Fr_pow(PFrElement r, PFrElement a, PFrElement b) {
    mpz_t ma;
    mpz_t mb;
    mpz_t mq;
    mpz_t mr;
    mpz_init(ma);
    mpz_init(mb);
    mpz_init(mq);
    mpz_init(mr);

    Fr_toMpz(ma, a);
    Fr_toMpz(mb, b);
    mpz_set_si(mq, MOD);
    mpz_powm(mr, ma, mb, mq);
    Fr_fromMpz(r, mr);

    mpz_clear(ma);
    mpz_clear(mb);
    mpz_clear(mq);
    mpz_clear(mr);
}
