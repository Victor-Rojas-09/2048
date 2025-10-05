use crate::board::*;

/// One line/column of the board
type Row = [u8; N];

pub fn eval(board: &Board) -> f32 {
    let mut sum = 0.0;
    for row in board.cells.iter() {
        sum += eval_row(row);
    }
    for col in board.transposed().cells.iter() {
        sum += eval_row(col);
    }
    sum
}

const NOT_LOST: f32 = 200_000f32;
const MONOTONICITY_WEIGHT: f32 = 47.0;
const EMPTY_WEIGHT: f32 = 270.0;
const ADJACENT_WEIGHT: f32 = 700.0;
const SUM_WEIGHT: f32 = 11.0;

fn eval_row(row: &Row) -> f32 {
    NOT_LOST
        + monotonicity(row) * MONOTONICITY_WEIGHT
        + empty(row) * EMPTY_WEIGHT
        + adjacent(row) * ADJACENT_WEIGHT
        + sum(row) * SUM_WEIGHT
}

fn empty(row: &Row) -> f32 {
    row.iter().filter(|&&cell| cell == 0).count() as f32
}

fn monotonicity(row: &Row) -> f32 {
    let mut left = 0;
    let mut right = 0;

    for i in 0..(N - 1) {
        let current = row[i];
        let next = row[i + 1];
        if current > next {
            left += i32::from(current).pow(4) - i32::from(next).pow(4);
        } else if next > current {
            right += i32::from(next).pow(4) - i32::from(current).pow(4);
        }
    }

    -left.min(right) as f32
}

fn adjacent(row: &Row) -> f32 {
    let mut adjacent_count = 0;
    let mut i = 0;

    while i < N - 1 {
        if row[i] != 0 && row[i] == row[i + 1] {
            adjacent_count += 1;
            i += 2;
        } else {
            i += 1;
        }
    }

    adjacent_count as f32
}

fn sum(row: &Row) -> f32 {
    -row.iter().map(|&v| POW_3_5_LOOKUP[v as usize]).sum::<f32>()
}

/// lookup table: `POW_3_5_LOOKUP[i]` is equal to `i^3.5` but faster to compute
const POW_3_5_LOOKUP: [f32; 18] = [
    0.0, 1.0, 11.313708, 46.765373, 128.0, 279.50848, 529.0898, 907.4927, 1448.1547, 2187.0,
    3162.2776, 4414.4277, 5985.968, 7921.396, 10267.107, 13071.318, 16384.0, 20256.818,
];
