use num_bigint::BigInt;

const P_M31: &str = "2147483647";
//const P_STR: &str = "21888242871839275222246405745257275088548364400416034343698204186575808495617";

pub struct UsefulConstants {
    p: BigInt,
}

impl Clone for UsefulConstants {
    fn clone(&self) -> Self {
        UsefulConstants { p: self.p.clone() }
    }
}

impl UsefulConstants {
    pub fn new(possible_prime: &String) -> UsefulConstants {
        let prime_to_use = if possible_prime.eq("m31") {
            P_M31
        } else {
            unreachable!()
        };

        UsefulConstants {
            p: BigInt::parse_bytes(prime_to_use.as_bytes(), 10).expect("can not parse p"),
        }
    }

    pub fn get_p(&self) -> &BigInt {
        &self.p
    }
}
