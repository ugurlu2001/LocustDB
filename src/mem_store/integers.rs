use bit_vec::BitVec;
use value::Val;
use mem_store::column::{ColumnData, ColIter};
use heapsize::HeapSizeOf;
use std::{u8, u16, u32, i64};
use num::traits::NumCast;
use engine::types::Type;


pub struct IntegerColumn {
    values: Vec<i64>,
}

impl IntegerColumn {
    pub fn new(mut values: Vec<i64>, min: i64, max: i64) -> Box<ColumnData> {
        if max - min <= u8::MAX as i64 {
            Box::new(IntegerOffsetColumn::<u8>::new(values, min))
        } else if max - min <= u16::MAX as i64 {
            Box::new(IntegerOffsetColumn::<u16>::new(values, min))
        } else if max - min <= u32::MAX as i64 {
            Box::new(IntegerOffsetColumn::<u32>::new(values, min))
        } else {
            values.shrink_to_fit();
            Box::new(IntegerColumn { values: values })
        }
    }
}

impl ColumnData for IntegerColumn {
    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self.values.iter().map(|&i| Val::Integer(i));
        ColIter::new(iter)
    }

    fn dump_untyped<'a>(&'a self, count: usize, offset: usize, buffer: &mut Vec<Val<'a>>) {
        for i in offset..(offset + count) {
            buffer.push(Val::from(self.values[i + offset]));
        }
    }

    fn collect_int<'a>(&'a self, count: usize, offset: usize, filter: &Option<BitVec>, buffer: &mut Vec<i64>) {
        match filter {
            &None => {
                for i in offset..(offset + count) {
                    buffer.push(self.values[i + offset]);
                }
            }
            &Some(ref bv) => {
                for (i, select) in bv.iter().enumerate() {
                    if select {
                        buffer.push(self.values[i + offset]);
                    }
                }
            }
        }
    }

    fn decoded_type(&self) -> Type { Type::I64 }
}

trait IntLike: NumCast + HeapSizeOf {}

impl IntLike for u8 {}

impl IntLike for u16 {}

impl IntLike for u32 {}

struct IntegerOffsetColumn<T: IntLike> {
    values: Vec<T>,
    offset: i64,
}

impl<T: IntLike> IntegerOffsetColumn<T> {
    fn new(values: Vec<i64>, offset: i64) -> IntegerOffsetColumn<T> {
        let mut encoded_vals = Vec::with_capacity(values.len());
        for v in values {
            encoded_vals.push(T::from(v - offset).unwrap());
        }
        IntegerOffsetColumn {
            values: encoded_vals,
            offset: offset,
        }
    }
}

impl<T: IntLike + Send + Sync> ColumnData for IntegerOffsetColumn<T> {
    fn iter<'a>(&'a self) -> ColIter<'a> {
        let offset = self.offset;
        let iter = self.values.iter().map(move |i| Val::Integer(i.to_i64().unwrap() + offset));
        ColIter::new(iter)
    }

    fn dump_untyped<'a>(&'a self, count: usize, offset: usize, buffer: &mut Vec<Val<'a>>) {
        for i in offset..(offset + count) {
            buffer.push(Val::from(self.values[i + offset].to_i64().unwrap() + self.offset));
        }
    }

    fn collect_int<'a>(&'a self, count: usize, offset: usize, filter: &Option<BitVec>, buffer: &mut Vec<i64>) {
        match filter {
            &None => {
                for i in offset..(offset + count) {
                    buffer.push(self.values[i + offset].to_i64().unwrap() + self.offset);
                }
            }
            &Some(ref bv) => {
                for (i, select) in bv.iter().enumerate() {
                    if select {
                        buffer.push(self.values[i + offset].to_i64().unwrap() + self.offset);
                    }
                }
            }
        }
    }

    fn decoded_type(&self) -> Type { Type::I64 }
}

impl HeapSizeOf for IntegerColumn {
    fn heap_size_of_children(&self) -> usize {
        self.values.heap_size_of_children()
    }
}

impl<T: IntLike> HeapSizeOf for IntegerOffsetColumn<T> {
    fn heap_size_of_children(&self) -> usize {
        self.values.heap_size_of_children()
    }
}
