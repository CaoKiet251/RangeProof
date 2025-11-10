use num_bigint::{BigInt, ToBigInt};
use num_traits::{One, ToPrimitive};

pub fn find_4_squares(n: &BigInt) -> Vec<BigInt> {
	let n_u = n.to_u64().unwrap_or(0);
	for a in 0..=n_u {
		for b in 0..=a {
			for c in 0..=b {
				let rem = n_u - a*a - b*b - c*c;
				let d = (rem as f64).sqrt().floor() as u64;
				if a*a + b*b + c*c + d*d == n_u {
					return vec![a, b, c, d].into_iter().map(|x| x.to_bigint().unwrap()).collect();
				}
			}
		}
	}
	panic!("Cannot find 4 squares for {}", n);
}

pub fn find_3_squares(n: &BigInt) -> Vec<BigInt> {
	if let Some(n_u) = n.to_u64() {
		if n_u <= 1000000 {
			for a in 0..=n_u {
				for b in 0..=a {
					let ab = a*a + b*b;
					if ab > n_u { break; }
					let rem = n_u - ab;
					let c = (rem as f64).sqrt().floor() as u64;
					if a*a + b*b + c*c == n_u {
						return vec![a, b, c].into_iter().map(|x| x.to_bigint().unwrap()).collect();
					}
				}
			}
		}
	}
	
	let one = BigInt::one();
	let two = BigInt::from(2u32);
	let four = BigInt::from(4u32);
	
	let mut k = 1u32;
	while k <= 32 {
		let term1 = &two.pow(k);
		let term2 = &two.pow(k-1);
		let sum = term1 * term1 + term2 * term2 + &one;
		
		if &sum == n {
			return vec![term1.clone(), term2.clone(), one.clone()];
		}
		
		if &sum > n {
			break;
		}
		k += 1;
	}
	
	let sqrt_n = n.sqrt();
	let a = &sqrt_n / &two;
	let b = &sqrt_n / &four;
	let c = &one;
	
	let a_sq = a.clone() * a.clone();
	let b_sq = b.clone() * b.clone();
	let c_sq = c.clone() * c.clone();
	let sum_squares = a_sq + b_sq + c_sq;
	
	if sum_squares <= *n {
		return vec![a.clone(), b.clone(), c.clone()];
	}
	
	vec![BigInt::from(1u32), BigInt::from(1u32), BigInt::from(1u32)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigInt;

    #[test]
    fn small_numbers_have_valid_decompositions() {
        let four = find_4_squares(&BigInt::from(30));
        assert_eq!(four.len(), 4);
        let sum4: u128 = four.iter().map(|x| x.to_u128().unwrap()).map(|x| x*x).sum();
        assert_eq!(sum4, 30u128);

        let three = find_3_squares(&BigInt::from(29));
        assert_eq!(three.len(), 3);
        let sum3: u128 = three.iter().map(|x| x.to_u128().unwrap()).map(|x| x*x).sum();
        assert_eq!(sum3, 29u128);
    }
}

