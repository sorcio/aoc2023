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
pub(crate) trait Unindentable {
    type Output: Borrow<Self>;
    fn unindent(&self) -> Self::Output;
}

impl Unindentable for str {
    type Output = String;
    fn unindent(&self) -> String {
        unindent::unindent(self)
    }
}

impl Unindentable for [u8] {
    type Output = Vec<u8>;
    fn unindent(&self) -> Vec<u8> {
        unindent::unindent_bytes(self)
    }
}

trait ParserOrNone<'s, T: ?Sized> {
    type Parsed;
    fn parse(self, input: &'s T) -> Self::Parsed;
}

impl<'s, F: FnOnce(&T) -> I, T: Unindentable + ?Sized + 's, I> ParserOrNone<'s, T> for F {
    type Parsed = I;
    fn parse(self, input: &'s T) -> Self::Parsed {
        self(input)
    }
}

impl<'s, T: Unindentable + ?Sized + 's> ParserOrNone<'s, T> for Option<()> {
    type Parsed = &'s T;
    fn parse(self, input: &'s T) -> Self::Parsed {
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
            $solver_name:ident => $result:expr
        ),+
        $(,)?
    ) => {
        #[cfg(test)]
        mod example_tests {
            $(
                #[test]
                fn $solver_name() {
                    use std::borrow::Borrow;
                    use $crate::testing::{CorrectResultTest, Unindentable};
                    #[allow(unused_variables)]
                    let example_data = $example_data.unindent();
                    $(
                        let example_data = $per_part_example_data.unindent();
                    )?
                    {
                    CorrectResultTest {
                        parser: $parser,
                        solver: super::$solver_name,
                        example: example_data.borrow(),
                        result: &$result,
                        marker: std::marker::PhantomData,
                    }.test();
                }
                }
            )*
        }
    };
    ($example_data:expr, $($solver_name:ident => $result:expr),+ $(,)?) => {
        example_tests! {
            parser: super::parse,
            $example_data,
            $($solver_name => $result),*
        }
    };
}

macro_rules! known_input_tests {
    (
        parser: $parser:expr,
        input: $input:expr,
        $(
            $solver_name:ident => $result:expr
        ),+
        $(,)?
    ) => {
        #[cfg(test)]
        mod known_input_tests {
            $(
                #[test]
                fn $solver_name() {
                    use std::borrow::Borrow;
                    use $crate::testing::{CorrectResultTest, Unindentable};
                    let example_data = $input.unindent();
                    {
                    CorrectResultTest {
                        parser: $parser,
                        solver: super::$solver_name,
                        example: example_data.borrow(),
                        result: &$result,
                        marker: std::marker::PhantomData,
                    }.test();
                }
                }
            )*
        }
    };
    (input: $input:expr, $($solver_name:ident => $result:expr),+ $(,)?) => {
        known_input_tests! {
            parser: super::parse,
            input: $input,
            $($solver_name => $result),*
        }
    };
}

pub(crate) use {example_tests, known_input_tests};
