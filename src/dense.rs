//! module for the dense matrix type

use std::ops::{IndexMut, Index, Mul, MulAssign};
use crate::matrix::{Matrix, MatrixError, RowOps, Concatenate};
use crate::numerics::Numeric;
use crate::utilities::ApproxEq;
use crate::MatrixT;

use std::fmt::{Display, Debug};

/// a dense matrix stores all the values of the matrix
/// a matrix is a vec with dimensional properties (m x n)
/// m the vertical length (rows)
/// n represents the horizontal length (columns)
/// it is stored as a row-major vector similarly to C
/// The matrix uses zero referencing
/// A dense matrix elements can be referenced by element
/// # indexing elements:
/// ```
/// # use numb_rs::{mat, Dense};
/// # fn main() {
/// let a = mat![
///     0, 1, 2;
///     3, 4, 5
/// ];
///
/// assert_eq!(a[[1, 2]], 5); // Direct access to element at 1, 2
/// assert_eq!(a[1], [3, 4, 5]); // access to row 1
/// assert_eq!(a[1][2], 5); // access to element 2 in row 1
/// # }
/// ```
/// # mutable indexing:
/// ```
/// # use numb_rs::{mat, Dense};
/// # fn main() {
/// let mut a = mat![
///     0, 1, 2;
///     3, 4, 5
/// ];
///
/// a[[1, 0]] = 6;
/// a[0][1] = -9;
///
/// assert_eq!(a[[1, 0]], 6);
/// assert_eq!(a[0], [0, -9, 2]);
/// assert_eq!(a[0][1], -9);
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Dense<T: Numeric> {
    /// a vector containing the Matrix data
    pub data: Vec<T>,
    /// number of rows
    pub m: usize,
    /// number of columns
    pub n: usize,
}

impl<T: Numeric> Matrix for Dense<T> {
    type Element = T;

    fn len(&self) -> usize {
        self.data.len()
    }

    fn size(&self) -> [usize; 2] {
        [self.m, self.n]
    }

    fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    fn into_vec(self) -> Vec<T> {
        self.data
    }
}

impl<T: Numeric> Default for Dense<T> {
    fn default() -> Self {
        Dense {
            data: Vec::new(),
            m: 1,
            n: 0,
        }
    }
}

impl<T: Numeric> Index<[usize; 2]> for Dense<T> {
    type Output = T;
    /// takes i, j returns the element
    fn index(&self, idx: [usize; 2]) -> &T {
        &self.data[idx[1] + idx[0] * self.n]
    }
}

impl<T: Numeric> Index<usize> for Dense<T> {
    type Output = [T];

    fn index(& self, index: usize) -> &Self::Output {
        let a = self.n * index;
        &self.data[a..a + self.n]
    }
}

impl<T: Numeric> IndexMut<[usize; 2]> for Dense<T> {
    /// takes i, j returns a mutable reference
    fn index_mut(&mut self, idx: [usize; 2]) -> &mut T {
        &mut self.data[idx[1] + idx[0] * self.n]
    }
}

impl<T: Numeric> IndexMut<usize> for Dense<T> {

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let a = self.n * index;
        &mut self.data[a..a + self.n]
    }
}

impl<T: Numeric> Display for Dense<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // closure to format each element
        let precision = f.precision().unwrap_or_else(|| 2);
        let format = |x: &T| format!("{:.*}", precision ,x);

        // first run through to find the max length of each formatted element
        // elements are stored in a vec as we go
        let mut strings: Vec<String> = vec![];
        let max = self.data
            .iter()
            .fold(0, |max: usize, x:&T| {
                let s = format(x);
                let disp_len = s.len();
                strings.push(s);
                if max > disp_len {max} else {disp_len}
            }) + 2;

        // iterate through the stored vector folding each formatted element into a final string
        // also adding a new line when each element divides evenly into the number of rows
        let string = strings.iter().enumerate().fold(
            "".to_string(), | mut s, (i, x)| {
                if i % self.n == 0 && i != 0 {s.push_str("\n")}
                format!("{}{:>width$}", s, x, width=max)
            });

        write!(f, "{}", string)
    }
}

impl<T: Numeric> Dense<T> {
    /// returns an empty matrix
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new matrix with a specific vector capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Dense {
            data: Vec::with_capacity(capacity),
            ..Dense::default()
        }
    }

    /// returns an identity matrix
    /// Will require a type
    pub fn eye(size: usize) -> Dense<T>
    where
        T: std::convert::From<u32>,
    {
        let mut v: Vec<T> = Vec::new();
        for i in 0..size {
            for j in 0..size{
                if i == j {
                    v.push(1.into());
                } else {
                    v.push(0.into());
                }
            }
        }
        Dense::<T> {
            data: v,
            m: size,
            n: size,
        }
    }

    /// swaps two elements in the vector
    /// This method only swaps the pointers similar to the vector implementation
    #[inline]
    fn swap(&mut self, a: [usize; 2], b: [usize; 2]) {
        unsafe {
            let pa: *mut T = &mut self[a];
            let pb: *mut T = &mut self[b];
            std::ptr::swap(pa, pb)
        }
    }

    /// this method returns self wrapped in a MatrixT struct to indicate that methods should index
    /// the transpose of the struct it does not perform any matrix
    pub fn t(&self) -> MatrixT<T> {
        MatrixT {
            /// a reference to the vector of the Matrix below
            data: &self.data,
            /// m is a reference to the 'n' column of the Matrix
            m: &self.n,
            /// n is a reference to the 'm' column of the Matrix
            n: &self.m,
        }
    }
}

impl<T: Numeric> RowOps<T> for Dense<T>{
    /// Scales all elements in a given row
    fn scale_row(&mut self, i: usize, scale: T){
        for j in 0..self.n {
            self[[i, j]] *= scale;
        }
    }

    /// adds one row to another with a scaling factor
    fn add_rows(&mut self, base: usize, row_to_add: usize, scale: T){
        for j in 0..self.n {
            let x = self[[row_to_add, j]] * scale;
            self[[base, j]] += x;
        }
    }

    /// swaps rows a and b
    fn swap_rows(&mut self, a: usize, b:usize) {
        assert!(a < self.m && b < self.m);
        for (j, _) in (0..self.n).enumerate() {
            self.swap([a, j], [b, j])
        }
    }
}

impl<T: Numeric> Concatenate<Dense<T>, T> for Dense<T> {
    fn concatenate(self, other: Dense<T>) -> Result<Dense<T>, MatrixError> {
        // check that matrices are compatible
        match self.m == other.m {
            true => {
                // create a matrix with a capacity
                let mut new: Dense<T> = Dense::with_capacity(
                    self.data.capacity() + other.data.capacity()
                );
                new.n = self.n + other.n;
                new.m = self.m;
                // TODO: Vectorise this loop

                // if we think of appending to a vector instead of a 2d array we might consider
                // that we wish to add a row starting at 'i' in the vector and push values
                // onto the new array

                for i in 0..self.m {
                    for j in 0..self.n {
                        new.data.push(self[[i, j]]);
                    }
                    for j in 0..other.n{
                        new.data.push(other[[i, j]])
                    }
                }

                Ok(new)
            }
            false => Err(MatrixError::Incompatibility),
        }
    }
}

pub struct MatrixIterator<'a, T: Numeric> {
    matrix: &'a Dense<T>,
    i: usize
}

impl<'a, T: Numeric> IntoIterator for &'a Dense<T> {
    type Item = &'a [T];
    type IntoIter = MatrixIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        MatrixIterator {
            matrix: self,
            i: 0
        }
    }
}

impl<'a, T: Numeric> Iterator for MatrixIterator<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < self.matrix.m {
            let out = &self.matrix[self.i];
            self.i += 1;
            Some(out)
        } else {
            None
        }
    }
}

/// multiplying a Matrix by a scalar of the same type
impl<T: Numeric> Mul<T> for Dense<T> {
    type Output = Self;

    fn mul(self, scalar: T) -> Self {
        let v: Vec<T> = self.data.into_iter().map(|x| x * scalar).collect();

        Dense { data: v, ..self }
    }
}

impl<T: Numeric> MulAssign<T> for Dense<T>{
    fn mul_assign(&mut self, scalar: T) {
        self.data.iter_mut().for_each(|x| *x *= scalar)
    }
}

/// Matrix multiplication returns the dot product
/// The matrices must have dimensions such that mn * nk = mk
/// This is a naive solution, there are more efficient computational methods tbd later
impl<T: Numeric> Mul<Dense<T>> for Dense<T> {
    type Output = Result<Self, MatrixError>;

    fn mul(self, other: Self) -> Self::Output {
        if self.n != other.m {
            Err(MatrixError::Incompatibility)
        } else {
            let mut out: Dense<T> = Dense::with_capacity(self.m * other.n);
            out.m = self.m;
            out.n = other.n;

            unsafe {
                out.data.set_len(out.m * out.n);
            }

            for i in 0..out.m {
                for j in 0..out.n {
                    out[[i, j]] = T::ZERO;
                    for k in 0..self.n {
                        out[[i, j]] += self[[i, k]] * other[[k, j]]
                    }
                }
            }
            Ok(out)
        }
    }
}

impl ApproxEq<Dense<f64>> for Dense<f64> {
    type Check = f64;

    fn approx_eq(&self, other: &Dense<f64>, tolerance: Self::Check) -> bool {
        if self.m != other.m || self.n != other.n {
            return false;
        }

        for i in 0..self.m {
            for j in 0..self.n {
                if (self[[i, j]] - other[[i, j]]).abs() > tolerance {
                    return false;
                }
            }
        }
        true
    }

    fn assert_approx_eq(&self, other: &Dense<f64>, tolerance: Self::Check) {
        if self.m != other.m || self.n != other.n {
            panic!(
                r#"assertion failed: Dimension Inequality
    left  m x n: `{:?}`x`{:?}`
    right m x n: `{:?}`x`{:?}`"#,
                self.m, other.m, self.n, other.n
            )
        }

        for i in 0..self.m {
            for j in 0..self.n {
                let delta = (self[[i, j]] - other[[i, j]]).abs();
                if delta > tolerance {
                    panic!(
                        r#"assertion failed at element [{:?}, {:?}]: ± `{:?}`
    left: `{:?}`
    right: `{:?}`
    delta = `{:?}`"#,
                        i,
                        j,
                        tolerance,
                        self[[i, j]],
                        other[[i, j]],
                        delta
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raw_matrix() {
        let a = Dense {
            data: vec![0, 1, 2, 3, 4, 5],
            n: 3,
            m: 2,
        };
        assert_eq!(a[[0, 0]], 0);
        assert_eq!(a[[0, 1]], 1);
        assert_eq!(a[[0, 2]], 2);
        assert_eq!(a[[1, 0]], 3);
        assert_eq!(a[[1, 1]], 4);
        assert_eq!(a[[1, 2]], 5);

        let i: Dense<u32> = Dense::eye(3);
        let eye3 = mat![
            1, 0, 0;
            0, 1, 0;
            0, 0, 1
        ];

        assert_eq!(i, eye3);

        let j: Dense<f64> = Dense::eye(2);
        j.assert_approx_eq(&mat![1., 0.; 0., 1.], std::f64::EPSILON)
    }

    #[test]
    fn macro_tests() {
        let a: Dense<f64> = mat!();
        assert!(a.is_empty());

        let b: Dense<u8> = mat![0;1;2;3;4];
        assert!(!b.is_empty());
        assert_eq!(b.m, 5);
        assert_eq!(b.n, 1);

        let c = mat![1];
        assert_eq!(c.len(), 1);

        let d = mat![0, 1, 2];
        assert_eq!(d.len(), 3);

        let e = mat![
            0, 1, 2;
            3, 4, 5
        ];

        assert_eq!(e.len(), 6);
        assert_eq!(e[[0, 0]], 0);
        assert_eq!(e[[0, 1]], 1);
        assert_eq!(e[[0, 2]], 2);
        assert_eq!(e[[1, 0]], 3);
        assert_eq!(e[[1, 1]], 4);
        assert_eq!(e[[1, 2]], 5);

        let f = mat![3 => 2, 2];
        assert_eq!(f[[0, 0]], 3);
        assert_eq!(f[[0, 1]], 3);
        assert_eq!(f[[1, 0]], 3);
        assert_eq!(f[[1, 1]], 3);
    }

    #[test]
    fn approx_matrix_test() {
        let a: Dense<f64> = mat![
            1., 2., 3.;
            0.000001, 0., 1000.
        ];

        let b: Dense<f64> = mat![
            1., 2., 3.;
            0.0000011, 0., 1000.
        ];

        assert!(&a.approx_eq(&b, 0.0000002));
    }

    #[test]
    fn swap() {
        let mut a: Dense<u32> = mat![1,2,3;4,5,6;7,8,9];
        a.swap([0, 0], [2, 2]);
        a.swap([0, 1], [2, 0]);
        assert_eq!(a.data, vec![9, 7, 3, 4, 5, 6, 2, 8, 1]);
        assert_eq!(a, mat![9,7,3;4,5,6;2,8,1])
    }

    #[test]
    fn row_swap() {
        let mut a: Dense<i32> = mat![1,2,3,4; 5,6,7,8; 9,10,11,12];
        a.swap_rows(0, 2);
        let b: Dense<i32> = mat![9,10,11,12; 5,6,7,8; 1,2,3,4];
        assert_eq!(a, b)
    }

    #[test]
    fn concatenate() {
        let a = mat![1, 2; 3, 4];
        let b = mat![5; 6];
        let ans = mat![1, 2, 5; 3, 4, 6];

        assert_eq!(a.concatenate(b).unwrap(), ans);

        let a = mat![1, 5; 2, 6; 3, 7; 4, 8];
        let b = mat![9, 13; 10, 14; 11, 15; 12, 16];
        let ans = mat![1, 5, 9, 13; 2, 6, 10, 14; 3, 7, 11, 15; 4, 8, 12, 16];

        assert_eq!(a.concatenate(b).unwrap(), ans);
    }

    mod ops {
        use super::*;

        #[test]
        fn scalar_mul() {
            let x = mat![1, 2; 3, 4];
            assert_eq!(x * 2, mat![2, 4; 6, 8]);

            let mut x = mat![0, 4; 8, 10];
            x *= 3;
            assert_eq!(x, mat![0, 12; 24, 30]);
        }

        #[test]
        fn row_mul() {
            let mut x = mat![0, 4; 8, 10];

            x.scale_row(1, 2);

            assert_eq!(mat![0, 4; 16, 20], x)
        }

        #[test]
        fn row_add() {
            let mut x = mat![1, 2; 8, 10];

            x.add_rows(1, 0, 2);

            assert_eq!(mat![1, 2; 10, 14], x)
        }

        #[test]
        fn matrix_mul() {
            let a = mat![ 1, 3, 5; 7, 4, 6];
            let b = mat![4, 5; 2, 8; 4, 1];
            let c = a * b;
            let ans = mat![30, 34; 60, 73];
            assert!(c.is_ok());
            assert_eq!(c.unwrap(), ans);
        }

        #[test]
        fn matrix_incompatibilities() {
            let a = mat![1, 2, 3];
            let b = mat![2, 3; 4, 5];
            let c = a * b;
            assert!(c.is_err());
            assert_eq!(c.unwrap_err(), MatrixError::Incompatibility)
        }
    }

    #[test]
    fn matrix_print() {

        let i = mat![
            1, 2, 3;
            4, 5, 6;
            7, 8, 9
        ];

        assert_eq!(
            format!("{}", i),
            "  1  2  3\n  4  5  6\n  7  8  9".to_string()
        );

        let f = mat![
            0.1, 2.34, 3.14;
            4.05, -5.2, -6.84;
            7.999, 8.0023, 9.99
        ];

        assert_eq!(
            format!("{:.3}", f),
            "   0.100   2.340   3.140\n   4.050  -5.200  -6.840\n   7.999   8.002   9.990".to_string()
        );

        assert_eq!(
            format!("{}", f),
            "   0.10   2.34   3.14\n   4.05  -5.20  -6.84\n   8.00   8.00   9.99".to_string()
        );
    }
}