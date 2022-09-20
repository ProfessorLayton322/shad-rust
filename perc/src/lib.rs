#![forbid(unsafe_code)]

////////////////////////////////////////////////////////////////////////////////

use rand::Rng;
use std::vec::Vec;

/// Represents a grid of boolean values.
pub struct BoolGrid {
    _width: usize,
    _height: usize,
    _grid: Vec<Vec<bool>>,
}

impl BoolGrid {
    /// Creates a new grid with all values initialized as `false`.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            _width: width,
            _height: height,
            _grid: vec![vec![false; height]; width],
        }
    }

    /// Creates a new grid with every value initialized randomly.
    ///
    /// # Arguments
    ///
    /// * `width` - grid width.
    /// * `height` - grid height.
    /// * `vacancy` - probability of any given value being equal
    /// to `false`.
    pub fn random(width: usize, height: usize, vacancy: f64) -> Self {
        let mut rng = rand::thread_rng();
        let mut return_item = Self {
            _width: width,
            _height: height,
            _grid: vec![vec![true; height]; width],
        };
        for i in 0..width {
            for j in 0..height {
                if rng.gen_range(0.0..1.0) < vacancy {
                    return_item._grid[i][j] = false;
                }
            }
        }
        return return_item;
    }

    /// Returns grid width.
    pub fn width(&self) -> usize {
        return self._width;
    }

    /// Returns grid height.
    pub fn height(&self) -> usize {
        return self._height;
    }

    pub fn in_range(&self, x: usize, y: usize) -> bool {
        if x >= self._width || y >= self._height {
            return false;
        }
        return true;
    }

    pub fn get_neighbours(&self, x: usize, y: usize) -> Vec<(usize, usize)> {
        if !self.in_range(x, y) {
            panic!("Cell not in range");
        }
        let mut return_item = Vec::new();
        if x > 0 {
            return_item.push((x - 1, y));
        }
        if y > 0 {
            return_item.push((x, y - 1));
        }
        if x + 1 < self._width {
            return_item.push((x + 1, y));
        }
        if y + 1 < self._height {
            return_item.push((x, y + 1));
        }
        return return_item;
    }

    /// Returns the current value of a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or return incorrect result).
    pub fn get(&self, x: usize, y: usize) -> bool {
        if !self.in_range(x, y) {
            panic!("Cell not in range");
        }
        return self._grid[x][y];
    }

    /// Sets a new value to a given cell.
    /// The caller must ensure that `x` and `y` are valid.
    ///
    /// # Arguments
    ///
    /// * `x` - must be >= 0 and < grid width.
    /// * `y` - must be >= 0 and < grid height.
    ///
    /// # Panics
    ///
    /// If `x` or `y` is out of bounds, this method may panic
    /// (or set value to some other unspecified cell).
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        if !self.in_range(x, y) {
            panic!("Cell not in range");
        }
        self._grid[x][y] = value;
    }
}

////////////////////////////////////////////////////////////////////////////////

/// Returns `true` if the given grid percolates. That is, if there is a path
/// from any cell with `y` == 0 to any cell with `y` == `height` - 1.
/// If the grid is empty (`width` == 0 or `height` == 0), it percolates.

pub fn dfs(x: usize, y: usize, grid: &BoolGrid, used: &mut Vec<Vec<bool>>) {
    if grid.get(x, y) {
        return;
    }
    if used[x][y] {
        return;
    }
    used[x][y] = true;
    for neighbour in grid.get_neighbours(x, y) {
        let (nx, ny) = neighbour;
        dfs(nx, ny, grid, used);
    }
}

pub fn percolates(grid: &BoolGrid) -> bool {
    if grid.height() == 0 || grid.width() == 0 {
        return true;
    }
    let mut used: Vec<Vec<bool>> = vec![vec![false; grid.height()]; grid.width()];
    for x in 0..grid.width() {
        dfs(x, 0, grid, &mut used);
    }
    for x in 0..grid.width() {
        if used[x][grid.height() - 1] {
            return true;
        }
    }
    return false;
}

////////////////////////////////////////////////////////////////////////////////

const N_TRIALS: u64 = 10000;

/// Returns an estimate of the probability that a random grid with given
/// `width, `height` and `vacancy` probability percolates.
/// To compute an estimate, it runs `N_TRIALS` of random experiments,
/// in each creating a random grid and checking if it percolates.
pub fn evaluate_probability(width: usize, height: usize, vacancy: f64) -> f64 {
    let mut perc_count = 0;
    for _ in 0..N_TRIALS {
        let grid = BoolGrid::random(width, height, vacancy);
        if percolates(&grid) {
            perc_count += 1;
        }
    }
    return perc_count as f64 / N_TRIALS as f64;
}
