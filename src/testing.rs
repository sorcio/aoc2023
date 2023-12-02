use std::ops::Deref;

#[allow(private_bounds)]
pub(crate) struct CorrectResultTest<'s, Parse, Solve, I, IR, O>
where
    Parse: ParserOrNone<'s, Parsed = I>,
    Solve: FnOnce(&IR) -> O,
    I: Deref<Target = IR>,
    IR: ?Sized,
    O: 'static,
{
    pub(crate) parser: Parse,
    pub(crate) solver: Solve,
    pub(crate) example: &'s str,
    pub(crate) result: &'static O,
}

trait ParserOrNone<'s> {
    type Parsed;
    fn parse(self, input: &'s str) -> Self::Parsed;
}

impl<F: FnOnce(&str) -> I, I> ParserOrNone<'_> for F {
    type Parsed = I;
    fn parse(self, input: &str) -> I {
        self(input)
    }
}

impl<'s> ParserOrNone<'s> for Option<()> {
    type Parsed = &'s str;
    fn parse(self, input: &'s str) -> &'s str {
        match self {
            Some(_) => panic!("parser should be a function or None"),
            None => input,
        }
    }
}

#[allow(private_bounds)]
impl<'s, Parse, Solve, I, IR, O> CorrectResultTest<'s, Parse, Solve, I, IR, O>
where
    Parse: ParserOrNone<'s, Parsed = I>,
    Solve: FnOnce(&IR) -> O,
    I: Deref<Target = IR>,
    IR: ?Sized,
    O: std::cmp::Eq + std::fmt::Debug + 'static,
{
    #[cfg_attr(not(test), allow(unused))]
    pub(crate) fn test(self) {
        assert_eq!(
            &(self.solver)(&self.parser.parse(self.example)),
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
            #[allow(unused)]
            const EXAMPLE_DATA: &str = $example_data;
            $(
                #[test]
                fn $solver_name() {
                    use $crate::testing::CorrectResultTest;
                    #[allow(unused_variables)]
                    let example_data = EXAMPLE_DATA;
                    $(
                        let example_data = $per_part_example_data;
                    )?
                    let example_data = unindent::unindent(example_data);
                    {
                    CorrectResultTest {
                        parser: $parser,
                        solver: super::$solver_name,
                        example: &example_data,
                        result: &$result,
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
