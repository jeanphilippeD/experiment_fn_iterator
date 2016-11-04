use std::iter::Map;
use std::ops::FnMut;
use std::ops::Range;
use std::os::raw::{c_int, c_uint};


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

pub fn new_index_call_iterator<FLen, F, T>
    (f_len: FLen,
     f: F)
     -> Box<ExactSizeIterator<Item = T>>
    where F: Fn(c_uint) -> T + 'static,
          FLen: Fn() -> c_uint,
{
    Box::new((0..f_len()).map(f))
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

pub fn new_index_call_iterator_check_positive<FLen, F, T>
    (f_len: FLen,
     f: F)
     -> Option<Box<ExactSizeIterator<Item = T>>>
    where F: Fn(c_int) -> T + 'static,
          FLen: Fn() -> c_int,
{
    let len = f_len();
    if len >= 0 {
        Some(Box::new((0..len).map(f)))
    } else {
        assert_eq!(len, -1); // only expect -1 as invalid
        None
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

pub fn new_index_call_iterator_u32<FLen, F>(f_len: FLen,
                                            f: F)
                                            -> Box<Iterator<Item = u32>>
    where F: Fn(c_uint) -> c_uint + 'static,
          FLen: Fn() -> c_uint,
{
    Box::new((0..f_len()).map(move |x| f(x)))
}

pub fn new_index_call_iter<F>(len: c_uint, f: F) -> Map<Range<u32>, F>
    where F: Fn(c_uint) -> c_uint,
{
    (0..len).map(f)
}

pub fn new_index_call_iter_boxed<F>(len: c_uint,
                                    f: F)
                                    -> Box<Iterator<Item = u32>>
    where F: Fn(c_uint) -> c_uint + 'static,
{
    Box::new((0..len).map(move |x| f(x)))
}


// fn(A) -> (A, A)
pub fn new_ret_closure() -> Box<Fn(u32) -> u32> {
    Box::new(move |x| x + 2)
}

pub fn new_index_call_iter_test() {}

pub fn cipher_iter<'a>(data: &'a Vec<u8>,
                       key: u8)
                       -> Box<Iterator<Item = u8> + 'a> {
    Box::new(data.iter().map(move |&p| p ^ key))
}

pub fn cipher_iter_with_data_and_key() -> Box<Iterator<Item = u8>> {
    let key: u8 = 10;
    Box::new((0..6).map(move |p| p ^ key))
}

#[cfg(test)]
mod tests {
    use std::os::raw::{c_int, c_uint};
    use super::*;

    #[test]
    fn it_works() {}

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


    // A Simpler version
    //

    struct SimpleTestIndexCallableProvider {
        cxtu: c_uint,
        cxti: c_int,
    }

    impl SimpleTestIndexCallableProvider {
        fn get_unsigned_children(self) -> Box<ExactSizeIterator<Item = i32>> {
            new_index_call_iterator(|| self.cxtu, // call FFI for length
                                    |x| x as i32 /* call FFI for item */)
        }

        fn get_signed_children
            (self)
             -> Option<Box<ExactSizeIterator<Item = i32>>> {
            new_index_call_iterator_check_positive(|| self.cxti, // call FFI for length
                                                   |x| x as i32 /* call FFI for item */)
        }
    }

    #[test]
    fn test_simple_index_call_iterator() {
        let provider = SimpleTestIndexCallableProvider {
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
    fn test_simple_optional_index_call_iterator() {
        let provider = SimpleTestIndexCallableProvider {
            cxti: 2,
            cxtu: 3,
        };
        let provider_no_children = SimpleTestIndexCallableProvider {
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

    // Helper code to develop
    //


    #[test]
    fn test_new_index_call_iterator() {
        assert_eq!(new_index_call_iterator(|| 0 as u32, |x| (x + 10))
                       .collect::<Vec<_>>(),
                   vec![]);
        assert_eq!(new_index_call_iterator(|| 3 as u32, |x| (x + 10))
                       .collect::<Vec<_>>(),
                   vec![10, 11, 12]);
    }

    #[test]
    fn test_new_index_call_iterator_u32() {
        assert_eq!(new_index_call_iterator_u32(|| 0, |x| x)
                       .collect::<Vec<_>>(),
                   vec![]);
        assert_eq!(new_index_call_iterator_u32(|| 3, |x| x)
                       .collect::<Vec<_>>(),
                   vec![0, 1, 2]);
    }


    #[test]
    fn test_new_index_call_iter() {
        assert_eq!(new_index_call_iter(0, |x| x).collect::<Vec<_>>(), vec![]);
        assert_eq!(new_index_call_iter(3, |x| x).collect::<Vec<_>>(),
                   vec![0, 1, 2]);
    }

    #[test]
    fn test_new_index_call_iter_boxed() {
        assert_eq!(new_index_call_iter_boxed(0, |x| x).collect::<Vec<_>>(),
                   vec![]);
        assert_eq!(new_index_call_iter_boxed(3, |x| x).collect::<Vec<_>>(),
                   vec![0, 1, 2]);
    }

    // #[test]
    // fn test_new_index_call_iter_with_closuer_boxed() {
    //     let data = vec![1, 2, 3];
    //     assert_eq!(cipher_iter(&data, 10).collect::<Vec<_>>(), vec![11, 8, 9]);
    // }

    #[test]
    fn test_new_ret_closure() {
        assert_eq!(vec![1, 2, 3]
                       .into_iter()
                       .map(|x| x + 1)
                       .collect::<Vec<_>>(),
                   vec![2, 3, 4]);
        assert_eq!(vec![1, 2, 3]
                       .into_iter()
                       .map(&*new_ret_closure())
                       .collect::<Vec<_>>(),
                   vec![3, 4, 5]);
    }

    #[test]
    fn test_cipher_iter() {
        let data = vec![1, 2, 3];
        assert_eq!(cipher_iter(&data, 10).collect::<Vec<_>>(), vec![11, 8, 9]);
    }

    #[test]
    fn test_cipher_iter_keep_data_and_key() {
        assert_eq!(cipher_iter_with_data_and_key().collect::<Vec<_>>(),
                   vec![10, 11, 8, 9, 14, 15]);
    }
}
