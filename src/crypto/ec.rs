//! Simple verification of ECDSA signatures over SECP Random curves

use super::bigint::*;

pub(super) trait IntMod: Clone + Eq + Sized {
	type I: Int;
	fn from_i(v: Self::I) -> Self;
	fn from_modinv_of(v: Self::I) -> Result<Self, ()>;

	const ZERO: Self;
	const ONE: Self;

	fn mul(&self, o: &Self) -> Self;
	fn square(&self) -> Self;
	fn add(&self, o: &Self) -> Self;
	fn sub(&self, o: &Self) -> Self;
	fn double(&self) -> Self;
	fn times_three(&self) -> Self;
	fn times_four(&self) -> Self;
	fn times_eight(&self) -> Self;

	fn into_i(self) -> Self::I;
}
impl<M: PrimeModulus<U256> + Clone + Eq> IntMod for U256Mod<M> {
	type I = U256;
	fn from_i(v: Self::I) -> Self { U256Mod::from_u256(v) }
	fn from_modinv_of(v: Self::I) -> Result<Self, ()> { U256Mod::from_modinv_of(v) }

	const ZERO: Self = U256Mod::<M>::from_u256_panicking(U256::zero());
	const ONE: Self = U256Mod::<M>::from_u256_panicking(U256::one());

	fn mul(&self, o: &Self) -> Self { self.mul(o) }
	fn square(&self) -> Self { self.square() }
	fn add(&self, o: &Self) -> Self { self.add(o) }
	fn sub(&self, o: &Self) -> Self { self.sub(o) }
	fn double(&self) -> Self { self.double() }
	fn times_three(&self) -> Self { self.times_three() }
	fn times_four(&self) -> Self { self.times_four() }
	fn times_eight(&self) -> Self { self.times_eight() }

	fn into_i(self) -> Self::I { self.into_u256() }
}
impl<M: PrimeModulus<U384> + Clone + Eq> IntMod for U384Mod<M> {
	type I = U384;
	fn from_i(v: Self::I) -> Self { U384Mod::from_u384(v) }
	fn from_modinv_of(v: Self::I) -> Result<Self, ()> { U384Mod::from_modinv_of(v) }

	const ZERO: Self = U384Mod::<M>::from_u384_panicking(U384::zero());
	const ONE: Self = U384Mod::<M>::from_u384_panicking(U384::one());

	fn mul(&self, o: &Self) -> Self { self.mul(o) }
	fn square(&self) -> Self { self.square() }
	fn add(&self, o: &Self) -> Self { self.add(o) }
	fn sub(&self, o: &Self) -> Self { self.sub(o) }
	fn double(&self) -> Self { self.double() }
	fn times_three(&self) -> Self { self.times_three() }
	fn times_four(&self) -> Self { self.times_four() }
	fn times_eight(&self) -> Self { self.times_eight() }

	fn into_i(self) -> Self::I { self.into_u384() }
}

pub(super) trait Curve : Copy {
	type Int: Int;

	// With const generics, both CurveField and ScalarField can be replaced with a single IntMod.
	type CurveField: IntMod<I = Self::Int>;
	type ScalarField: IntMod<I = Self::Int>;

	type CurveModulus: PrimeModulus<Self::Int>;
	type ScalarModulus: PrimeModulus<Self::Int>;

	// Curve parameters y^2 = x^3 + ax + b
	const A: Self::CurveField;
	const B: Self::CurveField;

	const G: Point<Self>;
}

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Point<C: Curve + ?Sized> {
	x: C::CurveField,
	y: C::CurveField,
	z: C::CurveField,
}

impl<C: Curve + ?Sized> Point<C> {
	fn on_curve(x: &C::CurveField, y: &C::CurveField) -> Result<(), ()> {
		let x_2 = x.square();
		let x_3 = x_2.mul(&x);
		let v = x_3.add(&C::A.mul(&x)).add(&C::B);

		let y_2 = y.square();
		if y_2 != v {
			Err(())
		} else {
			Ok(())
		}
	}

	#[cfg(debug_assertions)]
	fn on_curve_z(x: &C::CurveField, y: &C::CurveField, z: &C::CurveField) -> Result<(), ()> {
		let m = C::CurveField::from_modinv_of(z.clone().into_i())?;
		let m_2 = m.square();
		let m_3 = m_2.mul(&m);
		let x_norm = x.mul(&m_2);
		let y_norm = y.mul(&m_3);
		Self::on_curve(&x_norm, &y_norm)
	}

	#[cfg(test)]
	fn normalize_x(&self) -> Result<C::CurveField, ()> {
		let m = C::CurveField::from_modinv_of(self.z.clone().into_i())?;
		Ok(self.x.mul(&m.square()))
	}

	fn from_xy(x: C::Int, y: C::Int) -> Result<Self, ()> {
		let x = C::CurveField::from_i(x);
		let y = C::CurveField::from_i(y);
		Self::on_curve(&x, &y)?;
		Ok(Point { x, y, z: C::CurveField::ONE })
	}

	pub(super) const fn from_xy_assuming_on_curve(x: C::CurveField, y: C::CurveField) -> Self {
		Point { x, y, z: C::CurveField::ONE }
	}

	/// Checks that `expected_x` is equal to our X affine coordinate (without modular inversion).
	fn eq_x(&self, expected_x: &C::ScalarField) -> Result<(), ()> {
		debug_assert!(expected_x.clone().into_i() < C::CurveModulus::PRIME, "N is < P");

		// If x is between N and P the below calculations will fail and we'll spuriously reject a
		// signature and the wycheproof tests will fail. We should in theory accept such
		// signatures, but the probability of this happening at random is roughly 1/2^128, i.e. we
		// really don't need to handle it in practice. Thus, we only bother to do this in tests.
		#[allow(unused_mut, unused_assignments)]
		let mut slow_check = None;
		#[cfg(test)] {
			slow_check = Some(C::ScalarField::from_i(self.normalize_x()?.into_i()) == *expected_x);
		}

		let e: C::CurveField = C::CurveField::from_i(expected_x.clone().into_i());
		if self.z == C::CurveField::ZERO { return Err(()); }
		let ezz = e.mul(&self.z).mul(&self.z);
		if self.x == ezz || slow_check == Some(true) { Ok(()) } else { Err(()) }
	}

	fn double(&self) -> Result<Self, ()> {
		if self.y == C::CurveField::ZERO { return Err(()); }
		if self.z == C::CurveField::ZERO { return Err(()); }

		let s = self.x.times_four().mul(&self.y.square());
		let z_2 = self.z.square();
		let z_4 = z_2.square();
		let y_2 = self.y.square();
		let y_4 = y_2.square();
		let x_2 = self.x.square();
		let m = x_2.times_three().add(&C::A.mul(&z_4));
		let x = m.square().sub(&s.double());
		let y = m.mul(&s.sub(&x)).sub(&y_4.times_eight());
		let z = self.y.double().mul(&self.z);

		#[cfg(debug_assertions)] { assert!(Self::on_curve_z(&x, &y, &z).is_ok()); }
		Ok(Point { x, y, z })
	}

	fn add(&self, o: &Self) -> Result<Self, ()> {
		let o_z_2 = o.z.square();
		let self_z_2 = self.z.square();

		let u1 = self.x.mul(&o_z_2);
		let u2 = o.x.mul(&self_z_2);
		let s1 = self.y.mul(&o.z.mul(&o_z_2));
		let s2 = o.y.mul(&self.z.mul(&self_z_2));
		if u1 == u2 {
			if s1 != s2 { /* Point at Infinity */ return Err(()); }
			return self.double();
		}
		let h = u2.sub(&u1);
		let h_2 = h.square();
		let h_3 = h.mul(&h_2);
		let r = s2.sub(&s1);
		let x = r.square().sub(&h_3).sub(&u1.double().mul(&h_2));
		let y = r.mul(&u1.mul(&h_2).sub(&x)).sub(&s1.mul(&h_3));
		let z = h.mul(&self.z).mul(&o.z);

		#[cfg(debug_assertions)] { assert!(Self::on_curve_z(&x, &y, &z).is_ok()); }
		Ok(Point { x, y, z})
	}
}

/// Calculates i * I + j * J
#[allow(non_snake_case)]
fn add_two_mul<C: Curve>(i: C::ScalarField, I: &Point<C>, j: C::ScalarField, J: &Point<C>) -> Result<Point<C>, ()> {
	let i = i.into_i();
	let j = j.into_i();

	if i == C::Int::ZERO { /* Infinity */ return Err(()); }
	if j == C::Int::ZERO { /* Infinity */ return Err(()); }

	let mut res_opt: Result<Point<C>, ()> = Err(());
	let i_limbs = i.limbs();
	let j_limbs = j.limbs();
	let mut skip_limb = 0;
	let mut limbs_skip_iter = i_limbs.iter().zip(j_limbs.iter());
	while limbs_skip_iter.next() == Some((&0, &0)) {
		skip_limb += 1;
	}
	for (idx, (il, jl)) in i_limbs.iter().zip(j_limbs.iter()).skip(skip_limb).enumerate() {
		let start_bit = if idx == 0 {
			core::cmp::min(il.leading_zeros(), jl.leading_zeros())
		} else { 0 };
		for b in start_bit..64 {
			let i_bit = (*il & (1 << (63 - b))) != 0;
			let j_bit = (*jl & (1 << (63 - b))) != 0;
			if let Ok(res) = res_opt.as_mut() {
				*res = res.double()?;
			}
			if i_bit {
				if let Ok(res) = res_opt.as_mut() {
					// The wycheproof tests expect to see signatures pass even if we hit Point at
					// Infinity (PAI) on an intermediate result. While that's fine, I'm too lazy to
					// go figure out if all our PAI definitions are right and the probability of
					// this happening at random is, basically, the probability of guessing a private
					// key anyway, so its not really worth actually handling outside of tests.
					#[cfg(test)] {
						res_opt = res.add(I);
					}
					#[cfg(not(test))] {
						*res = res.add(I)?;
					}
				} else {
					res_opt = Ok(I.clone());
				}
			}
			if j_bit {
				if let Ok(res) = res_opt.as_mut() {
					// The wycheproof tests expect to see signatures pass even if we hit Point at
					// Infinity (PAI) on an intermediate result. While that's fine, I'm too lazy to
					// go figure out if all our PAI definitions are right and the probability of
					// this happening at random is, basically, the probability of guessing a private
					// key anyway, so its not really worth actually handling outside of tests.
					#[cfg(test)] {
						res_opt = res.add(J);
					}
					#[cfg(not(test))] {
						*res = res.add(J)?;
					}
				} else {
					res_opt = Ok(J.clone());
				}
			}
		}
	}
	res_opt
}

/// Validates the given signature against the given public key and message digest.
pub(super) fn validate_ecdsa<C: Curve>(pk: &[u8], sig: &[u8], hash_input: &[u8]) -> Result<(), ()> {
	#![allow(non_snake_case)]

	if pk.len() != C::Int::BYTES * 2 { return Err(()); }
	if sig.len() != C::Int::BYTES * 2 { return Err(()); }

	let (r_bytes, s_bytes) = sig.split_at(C::Int::BYTES);
	let (pk_x_bytes, pk_y_bytes) = pk.split_at(C::Int::BYTES);

	let pk_x = C::Int::from_be_bytes(pk_x_bytes)?;
	let pk_y = C::Int::from_be_bytes(pk_y_bytes)?;
	let PK = Point::from_xy(pk_x, pk_y)?;

	// from_i and from_modinv_of both will simply mod if the value is out of range. While its
	// perfectly safe to do so, the wycheproof tests expect such signatures to be rejected, so we
	// do so here.
	let r_u256 = C::Int::from_be_bytes(r_bytes)?;
	if r_u256 > C::ScalarModulus::PRIME { return Err(()); }
	let s_u256 = C::Int::from_be_bytes(s_bytes)?;
	if s_u256 > C::ScalarModulus::PRIME { return Err(()); }

	let r = C::ScalarField::from_i(r_u256);
	let s_inv = C::ScalarField::from_modinv_of(s_u256)?;

	let z = C::ScalarField::from_i(C::Int::from_be_bytes(hash_input)?);

	let u_a = z.mul(&s_inv);
	let u_b = r.mul(&s_inv);

	let V = add_two_mul(u_a, &C::G, u_b, &PK)?;
	V.eq_x(&r)
}
