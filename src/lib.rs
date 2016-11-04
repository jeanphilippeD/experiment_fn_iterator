
use std::os::raw::{c_int, c_uint};
use std::ops::FnMut;
use std::ops::Range;
use std::iter::Map;


/// Provide index results.
pub trait IndexCallable {
    /// Item type for Iterator trait.
    type Item;

    /// Type reprensenting the number of items.
    /// If signed, only create iterator if positive.
    type ItemNum;

    /// Call the function retreiving number of items.
    fn fetch_item_num(&self) -> Self::ItemNum;

    /// Call the function retreiving the item for the index idx.
    /// This will always be called with 0 <= idx < num.
    /// num will always be the value converted from fetch_item_num.
    fn fetch_item(&mut self,
                  idx: Self::ItemNum,
                  num: Self::ItemNum)
                  -> Self::Item;
}

/// Tag for unsigned integer type
pub trait UnsignedIndexable {
    /// Convert from ItemNum index
    fn from_index(self) -> usize;
}

impl UnsignedIndexable for c_uint {
    fn from_index(self) -> usize {
        self as usize
    }
}

/// Tag for signed integer type
pub trait SignedIndexable {
    /// Convert from ItemNum index
    fn from_index(self) -> isize;
}

impl SignedIndexable for c_int {
    fn from_index(self) -> isize {
        self as isize
    }
}

/// Convert usize index to ItemNum
pub trait Indexable {
    /// Convert to ItemNum index
    fn as_index(idx: usize) -> Self;
}

impl Indexable for c_uint {
    fn as_index(idx: usize) -> c_uint {
        idx as c_uint
    }
}

impl Indexable for c_int {
    fn as_index(idx: usize) -> c_int {
        idx as c_int
    }
}


/// An iterator for a type's template arguments
pub struct IndexCallIterator<CxtT> {
    cxt: CxtT,
    length: usize,
    index: usize,
}

impl<CxtT: IndexCallable> IndexCallIterator<CxtT>
    where CxtT::ItemNum: UnsignedIndexable,
{
    fn new(cxt: CxtT) -> IndexCallIterator<CxtT> {
        let len: usize = cxt.fetch_item_num().from_index();
        IndexCallIterator {
            cxt: cxt,
            length: len,
            index: 0,
        }
    }
}

impl<CxtT: IndexCallable> IndexCallIterator<CxtT>
    where CxtT::ItemNum: SignedIndexable,
{
    fn new_check_positive(cxt: CxtT) -> Option<IndexCallIterator<CxtT>> {
        let len: isize = cxt.fetch_item_num().from_index();
        if len >= 0 {
            Some(IndexCallIterator {
                cxt: cxt,
                length: len as usize,
                index: 0,
            })
        } else {
            assert_eq!(len, -1); // only expect -1 as invalid
            None
        }
    }
}

impl<CxtT: IndexCallable> Iterator for IndexCallIterator<CxtT>
    where CxtT::ItemNum: Indexable,
{
    type Item = CxtT::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.length {
            let idx = self.index;
            self.index += 1;
            Some(self.cxt.fetch_item(CxtT::ItemNum::as_index(idx),
                                     CxtT::ItemNum::as_index(self.length)))
        } else {
            None
        }
    }
}

impl<CxtT: IndexCallable> ExactSizeIterator for IndexCallIterator<CxtT>
    where CxtT::ItemNum: Indexable,
{
    fn len(&self) -> usize {
        assert!(self.index <= self.length);
        self.length - self.index
    }
}

    // fn map<B, F>(self, f: F) -> Map<Self, F> where
    //     Self: Sized, F: FnMut(Self::Item) -> B,
    // {
    //     Map{iter: self, f: f}
    // }

fn new_index_call_iter<F>(len: c_uint, f: F) -> Map<Range<u32>, F>
where F: FnMut(c_uint) -> c_uint
{
    (0..len).map(f)
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }


    use std::os::raw::{c_int, c_uint};
    use super::*;

    struct TestIndexCallableProvider {
        cxtu: c_uint,
        cxti: c_int,
    }

    impl TestIndexCallableProvider {
        fn get_unsigned_children(self) -> IndexCallIterator<TestIndexCallable> {
            let idx_callable = TestIndexCallable {
                cxt: self.cxtu,
            };
            IndexCallIterator::new(idx_callable)
        }

        fn get_signed_children
            (self)
             -> Option<IndexCallIterator<TestIndexCallableOption>> {
            let idx_callable = TestIndexCallableOption {
                cxt: self.cxti,
            };
            IndexCallIterator::new_check_positive(idx_callable)
        }
    }

    struct TestIndexCallable {
        cxt: c_uint, // FFI function context
    }

    impl IndexCallable for TestIndexCallable {
        type Item = i32;
        type ItemNum = c_uint;

        fn fetch_item_num(&self) -> Self::ItemNum {
            self.cxt // call specific FFI function
        }

        fn fetch_item(&mut self,
                      idx: Self::ItemNum,
                      num: Self::ItemNum)
                      -> Self::Item {
            assert!(idx < num);
            idx as i32// call specific FFI function
        }
    }


    struct TestIndexCallableOption {
        cxt: c_int, // FFI function context
    }

    impl IndexCallable for TestIndexCallableOption {
        type Item = i32;
        type ItemNum = c_int;

        fn fetch_item_num(&self) -> Self::ItemNum {
            return self.cxt; // call specific FFI function
        }

        fn fetch_item(&mut self,
                      idx: Self::ItemNum,
                      num: Self::ItemNum)
                      -> Self::Item {
            assert!(idx < num);
            idx as i32 // call specific FFI function
        }
    }

    #[test]
    fn test_index_call_iterator() {
        let provider = TestIndexCallableProvider {
            cxti: 3,
            cxtu: 2,
        };

        let values = provider.get_unsigned_children();
        let len = values.len();
        let collected = values.collect::<Vec<_>>();

        assert_eq!(collected, vec![0, 1]);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_optional_index_call_iterator() {
        let provider = TestIndexCallableProvider {
            cxti: 2,
            cxtu: 3,
        };
        let provider_no_children = TestIndexCallableProvider {
            cxti: -1,
            cxtu: 3,
        };

        let values = provider.get_signed_children();
        let not_values = provider_no_children.get_signed_children();
        let len = values.as_ref().map(|x| x.len());
        let collected = values.map(|x| x.collect::<Vec<_>>());

        assert_eq!(collected, Some(vec![0, 1]));
        assert_eq!(len, Some(2));
        assert!(not_values.is_none());
    }

    #[test]
    fn test_new_index_call_iter(){
        //new_index_call_iter(0, |x| x)
    }
}
