#[derive(Debug, PartialEq, Eq)]
// generic struct
struct GenericStruct<'a, T>
where
    T: Iterator<Item = u8>,
{
    f: &'a mut T,
}

// impl for generic struct
impl<'a, T> Iterator for GenericStruct<'a, T>
where
    T: Iterator<Item = u8>,
{
    type Item = Option<u8>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        self.f.next().map(Some)
    }
}

#[allow(unused)]
// make sure we can locate functions
fn iter<F, T>(_: impl Iterator<Item = F>) -> Option<String>
where
    F: AsRef<String> + ToString + std::str::FromStr,
{
    None
}

// make sure looking inside a module works
mod module {
    #[derive(Debug, PartialEq, Eq)]
    // struct with the same name in a different module
    struct GenericStruct<'a, T>
    where
        T: Iterator<Item = u64>,
    {
        f: &'a mut T,
    }

    // a little different impl to identify
    impl<'a, T> Iterator for GenericStruct<'a, T>
    where
        T: Iterator<Item = u64>,
    {
        type Item = u64;

        fn next(&mut self) -> Option<<Self as Iterator>::Item> {
            self.f.next()
        }
    }

    #[allow(unused)]
    // do the same with functions, slightly distinctive
    fn iter<F, T>(_: impl Iterator<Item = F>) -> Option<String>
    where
        F: AsRef<String> + ToString + std::str::FromStr,
    {
        None
    }
}
