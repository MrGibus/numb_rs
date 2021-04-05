/// With a focus on being concise: This macro will create a matrix using syntax similar to matlab
/// A semicolon ';' represends a new matrix row
/// # example 1:
/// ```
/// # use numb_rs::{mat, Dense};
/// # fn main() {
/// let a = mat![
///     0, 1, 2;
///     3, 4, 5
/// ];
/// # }
/// ```
/// will provide a 3x2 matrix as specified
///
/// # example 2:
/// It's also possible to initialise a matrix with a given value
/// This uses a different syntax to standard rust code due to a semicolon being used to denote a
/// row change. for instance:
/// ```
/// let x = [0.;5];
/// ```
/// is translated to:
/// ```
/// # use numb_rs::{mat, Dense};
/// # fn main() {
/// let x = mat![0. => 5, 1];
/// # }
/// ```
/// where 5, 1 represent m and n, i.e. the row and column lengths respectively
///
#[macro_export]
macro_rules! mat {
    // empty
    () => {
        Dense::new()
    };
    // standard
    ($($($item:expr),+);+) => {{
        let mut v = Vec::new();
        // underscored to surpress warnings
        let mut _n;
        let mut m = 0;
        $(
            _n = 0;
            $({
                v.push($item);
                _n += 1;
            })*
            m += 1;
        )*
        Dense{
            data: v,
            n: _n,
            m,
        }
    }};
    // fills an array with a value
    ($val:expr => $m: expr, $n: expr) => {{
        let mut v = Vec::new();
        for _ in 0..($m * $n) {
            v.push($val)
        }
        Dense {
            data: v,
            m: $m,
            n: $n,
        }
    }}
}