use std::{borrow::Borrow, marker::PhantomData};

#[allow(private_bounds)]
pub(crate) struct CorrectResultTest<'s, Parse, Solve, T, I, O>
where
    Parse: ParserOrNone<'s, T>,
    I: ?Sized,
    T: ?Sized,
    O: 'static,
{
    pub(crate) parser: Parse,
    pub(crate) solver: Solve,
    pub(crate) example: &'s T,
    pub(crate) result: &'static O,
    pub(crate) marker: PhantomData<I>,
}

trait ParserOrNone<'s, T: ?Sized> {
    type Parsed;
    fn parse(self, input: &'s T) -> Self::Parsed;
}

impl<F: FnOnce(&str) -> I, I> ParserOrNone<'_, str> for F {
    type Parsed = I;
    fn parse(self, input: &str) -> I {
        self(&unindent::unindent(input))
    }
}

impl<F: FnOnce(&[u8]) -> I, I> ParserOrNone<'_, [u8]> for F {
    type Parsed = I;
    fn parse(self, input: &[u8]) -> I {
        self(&unindent::unindent_bytes(input))
    }
}

impl<'s, T: ?Sized + 's> ParserOrNone<'s, T> for Option<()> {
    type Parsed = &'s T;
    fn parse(self, input: &'s T) -> &'s T {
        match self {
            Some(_) => panic!("parser should be a function or None"),
            None => input,
        }
    }
}

#[allow(private_bounds)]
impl<'s, Parse, Solve, T, I, O> CorrectResultTest<'s, Parse, Solve, T, I, O>
where
    Parse: ParserOrNone<'s, T>,
    Solve: FnOnce(&I) -> O,
    Parse::Parsed: Borrow<I>,
    T: ?Sized,
    I: ?Sized,
    O: std::cmp::Eq + std::fmt::Debug + 'static,
{
    #[cfg_attr(not(test), allow(unused))]
    pub(crate) fn test(self) {
        assert_eq!(
            &(self.solver)(self.parser.parse(self.example).borrow()),
            self.result
        );
    }
}

macro_rules! example_tests {
    (
        parser: $parser:expr,
        $example_data:expr,
        $(
            $($per_part_example_data:literal,)?
            $solver_name:ident => $result:expr),
        *) => {
        #[cfg(test)]
        mod example_tests {
            $(
                #[test]
                fn $solver_name() {
                    use $crate::testing::CorrectResultTest;
                    #[allow(unused_variables)]
                    let example_data = $example_data;
                    $(
                        let example_data = $per_part_example_data;
                    )?
                    {
                    CorrectResultTest {
                        parser: $parser,
                        solver: super::$solver_name,
                        example: example_data,
                        result: &$result,
                        marker: std::marker::PhantomData,
                    }.test();
                }
                }
            )*
        }
    };
    ($example_data:expr, $($solver_name:ident => $result:expr),*) => {
        example_tests! {
            parser: super::parse,
            $example_data,
            $($solver_name => $result),*
        }
    };
}

pub(crate) use example_tests;
