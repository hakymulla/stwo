use std::iter::zip;
use std::ops::{Add, Deref, Mul, Neg, Sub};

use num_traits::Zero;

use crate::core::fields::Field;

/// Univariate polynomial stored as coefficients in the monomial basis.
#[derive(Debug, Clone)]
pub struct UnivariatePoly<F: Field>(Vec<F>);

impl<F: Field> UnivariatePoly<F> {
    pub fn new(coeffs: Vec<F>) -> Self {
        let mut polynomial = Self(coeffs);
        polynomial.truncate_leading_zeros();
        polynomial
    }

    pub fn eval_at_point(&self, x: F) -> F {
        horner_eval(&self.0, x)
    }

    // <https://en.wikibooks.org/wiki/Algorithm_Implementation/Mathematics/Polynomial_interpolation>
    pub fn interpolate_lagrange(xs: &[F], ys: &[F]) -> Self {
        assert_eq!(xs.len(), ys.len());

        let mut coeffs = Self::zero();

        for (i, (&xi, &yi)) in zip(xs, ys).enumerate() {
            let mut prod = yi;

            for (j, &xj) in xs.iter().enumerate() {
                if i != j {
                    prod /= xi - xj;
                }
            }

            let mut term = Self::new(vec![prod]);

            for (j, &xj) in xs.iter().enumerate() {
                if i != j {
                    term = term * (Self::x() - Self::new(vec![xj]));
                }
            }

            coeffs = coeffs + term;
        }

        coeffs.truncate_leading_zeros();

        coeffs
    }

    pub fn degree(&self) -> usize {
        let mut coeffs = self.0.iter().rev();
        let _ = (&mut coeffs).take_while(|v| v.is_zero());
        coeffs.len().saturating_sub(1)
    }

    fn x() -> Self {
        Self(vec![F::zero(), F::one()])
    }

    fn truncate_leading_zeros(&mut self) {
        while self.0.last() == Some(&F::zero()) {
            self.0.pop();
        }
    }
}

impl<F: Field> From<F> for UnivariatePoly<F> {
    fn from(value: F) -> Self {
        Self::new(vec![value])
    }
}

impl<F: Field> Mul<F> for UnivariatePoly<F> {
    type Output = Self;

    fn mul(mut self, rhs: F) -> Self {
        self.0.iter_mut().for_each(|coeff| *coeff *= rhs);
        self
    }
}

impl<F: Field> Mul for UnivariatePoly<F> {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self {
        if self.is_zero() || rhs.is_zero() {
            return Self::zero();
        }

        self.truncate_leading_zeros();
        rhs.truncate_leading_zeros();

        let mut res = vec![F::zero(); self.0.len() + rhs.0.len() - 1];

        for (i, coeff_a) in self.0.into_iter().enumerate() {
            for (j, &coeff_b) in rhs.0.iter().enumerate() {
                res[i + j] += coeff_a * coeff_b;
            }
        }

        Self::new(res)
    }
}

impl<F: Field> Add for UnivariatePoly<F> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let n = self.0.len().max(rhs.0.len());
        let mut res = Vec::new();

        for i in 0..n {
            res.push(match (self.0.get(i), rhs.0.get(i)) {
                (Some(&a), Some(&b)) => a + b,
                (Some(&a), None) | (None, Some(&a)) => a,
                _ => unreachable!(),
            })
        }

        Self(res)
    }
}

impl<F: Field> Sub for UnivariatePoly<F> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self + (-rhs)
    }
}

impl<F: Field> Neg for UnivariatePoly<F> {
    type Output = Self;

    fn neg(self) -> Self {
        Self(self.0.into_iter().map(|v| -v).collect())
    }
}

impl<F: Field> Zero for UnivariatePoly<F> {
    fn zero() -> Self {
        Self(vec![])
    }

    fn is_zero(&self) -> bool {
        self.0.iter().all(F::is_zero)
    }
}

impl<F: Field> Deref for UnivariatePoly<F> {
    type Target = [F];

    fn deref(&self) -> &[F] {
        &self.0
    }
}

/// Evaluates univariate polynomial using [Horner's method].
///
/// [Horner's method]: https://en.wikipedia.org/wiki/Horner%27s_method
pub fn horner_eval<F: Field>(coeffs: &[F], x: F) -> F {
    coeffs
        .iter()
        .rfold(F::zero(), |acc, &coeff| acc * x + coeff)
}

#[cfg(test)]
mod tests {
    use std::iter::zip;

    use super::{horner_eval, UnivariatePoly};
    use crate::core::fields::m31::BaseField;
    use crate::core::fields::FieldExpOps;

    #[test]
    fn lagrange_interpolation_works() {
        let xs = [5, 1, 3, 9].map(BaseField::from);
        let ys = [1, 2, 3, 4].map(BaseField::from);

        let poly = UnivariatePoly::interpolate_lagrange(&xs, &ys);

        for (x, y) in zip(xs, ys) {
            assert_eq!(poly.eval_at_point(x), y, "mismatch for x={x}");
        }
    }

    #[test]
    fn horner_eval_works() {
        let coeffs = [BaseField::from(9), BaseField::from(2), BaseField::from(3)];
        let x = BaseField::from(7);

        let eval = horner_eval(&coeffs, x);

        assert_eq!(eval, coeffs[0] + coeffs[1] * x + coeffs[2] * x.square());
    }
}
