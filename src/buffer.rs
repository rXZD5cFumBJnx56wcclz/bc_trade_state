use std::{
    ops::{Deref, Index},
    slice::SliceIndex,
};

use bc_utils::other::{roll_slice1, transpose, transpose_set};

#[derive(Clone, PartialEq, Debug)]
pub struct Buffer(pub Vec<Vec<f64>>);

impl Buffer {
    pub fn new(src: Vec<Vec<f64>>) -> Self {
        Self(src)
    }
}

impl Buffer {
    pub fn update(
        &mut self,
        src: Vec<f64>,
    ) {
        roll_slice1(&mut self.0, -1);
        let l = self.0.len() - 1;
        self.0[l] = src;
    }
    pub fn update_extend(
        &mut self,
        src: &[Vec<f64>],
    ) {
        roll_slice1(&mut self.0, -(src.len() as i32));
        for _ in 0..src.len() {
            self.0.pop();
        }
        self.0.extend_from_slice(src);
    }
}

impl Buffer {
    pub fn iter(&self) -> impl Iterator<Item = &Vec<f64>> {
        self.0.iter()
    }
    pub fn first(&self) -> Option<&Vec<f64>> {
        self.0.first()
    }
    pub fn last(&self) -> Option<&Vec<f64>> {
        self.0.last()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn as_slice(&self) -> &[Vec<f64>] {
        self.0.as_slice()
    }
    pub fn transpose(self) -> Self {
        Self(transpose(self.0))
    }
    pub fn transpose_set(&mut self) {
        transpose_set(&mut self.0);
    }
}

impl<T: SliceIndex<[Vec<f64>]>> Index<T> for Buffer {
    type Output = T::Output;
    fn index(
        &self,
        index: T,
    ) -> &Self::Output {
        &self.0[index]
    }
}

impl Extend<Vec<f64>> for Buffer {
    fn extend<T: IntoIterator<Item = Vec<f64>>>(
        &mut self,
        iter: T,
    ) {
        self.0.extend(iter);
    }
}

impl Deref for Buffer {
    type Target = [Vec<f64>];
    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}

pub trait ToBuff {
    fn to_buff(&self) -> Buffer;
}

impl ToBuff for [Vec<f64>] {
    fn to_buff(&self) -> Buffer {
        Buffer(self.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bc_utils::other::roll_slice1;

    use crate::prelude_tests::prelude::*;

    static BF: LazyLock<fn() -> Buffer> = LazyLock::new(|| || Buffer::new(SRC.to_vec()));

    #[test]
    fn update_res_1() {
        let mut bf = BF();
        let mut res = BF();
        bf.update(SRC_EL.to_vec());
        roll_slice1(&mut res.0, -1);
        let l = res.0.len() - 1;
        res.0[l] = SRC_EL.to_vec();
        assert_eq_pr!(bf, res);
    }

    #[test]
    fn update_extend_res_1() {
        let mut bf = BF();
        let mut res = BF();
        bf.update_extend(&SRC);
        roll_slice1(&mut res.0, -(SRC.len() as i32));
        for _ in 0..SRC.len() {
            res.0.pop();
        }
        res.0.extend_from_slice(&SRC);
        assert_eq_pr!(bf, res);
    }
}
