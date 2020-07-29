#[derive(Debug, PartialEq, Clone)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

pub(self) mod parsers {
    use crate::parser::SExpr;
    use nom::character::complete::char;
    use nom::character::complete::multispace0;
    use nom::combinator::map;
    use nom::error::context;
    use nom::sequence::delimited;
    use nom::sequence::preceded;
    use nom::sequence::terminated;

    pub fn atom(i: &str) -> nom::IResult<&str, SExpr> {
        map(nom::bytes::complete::is_not("( \t\n)"), |atom: &str| {
            SExpr::Atom(atom.to_string())
        })(i)
    }

    pub fn atoms(i: &str) -> nom::IResult<&str, SExpr> {
        map(nom::multi::many0(preceded(multispace0, atom)), |atoms| {
            SExpr::List(atoms)
        })(i)
    }

    pub fn list(i: &str) -> nom::IResult<&str, SExpr> {
        delimited(
            context("opening paren", char('(')),
            atoms,
            context("opening paren", char(')')),
        )(i)
    }

    pub fn many(i: &str) -> nom::IResult<&str, Vec<SExpr>> {
        nom::multi::many0(terminated(preceded(multispace0, list), multispace0))(i)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_many() {
            assert_eq!(
                many("(assets ./abcd.css)\n\n\t(hello world)"),
                Ok((
                    "",
                    vec![
                        SExpr::List(vec![
                            SExpr::Atom("assets".to_string()),
                            SExpr::Atom("./abcd.css".to_string()),
                        ]),
                        SExpr::List(vec![
                            SExpr::Atom("hello".to_string()),
                            SExpr::Atom("world".to_string()),
                        ])
                    ]
                ))
            );
        }

        #[test]
        fn test_list() {
            assert_eq!(
                list("(assets ./abcd.css)"),
                Ok((
                    "",
                    SExpr::List(vec![
                        SExpr::Atom("assets".to_string()),
                        SExpr::Atom("./abcd.css".to_string()),
                    ])
                ))
            );
            assert_eq!(
                list("(assets ./abcd.css logo.png)"),
                Ok((
                    "",
                    SExpr::List(vec![
                        SExpr::Atom("assets".to_string()),
                        SExpr::Atom("./abcd.css".to_string()),
                        SExpr::Atom("logo.png".to_string()),
                    ])
                ))
            );
        }

        #[test]
        fn test_atoms() {
            assert_eq!(
                atoms("assets ./abcd.css"),
                Ok((
                    "",
                    SExpr::List(vec![
                        SExpr::Atom("assets".to_string()),
                        SExpr::Atom("./abcd.css".to_string()),
                    ])
                ))
            );
            assert_eq!(
                atoms("assets ./abcd.css logo.png"),
                Ok((
                    "",
                    SExpr::List(vec![
                        SExpr::Atom("assets".to_string()),
                        SExpr::Atom("./abcd.css".to_string()),
                        SExpr::Atom("logo.png".to_string()),
                    ])
                ))
            );
        }

        #[test]
        fn test_atom() {
            assert_eq!(
                atom("./abcd.css efg"),
                Ok((" efg", SExpr::Atom("./abcd.css".to_string())))
            );
            assert_eq!(
                atom("abcd efg"),
                Ok((" efg", SExpr::Atom("abcd".to_string())))
            );
            assert_eq!(
                atom("abcd\tefg"),
                Ok(("\tefg", SExpr::Atom("abcd".to_string())))
            );
            assert_eq!(
                atom(" abcdefg"),
                Err(nom::Err::Error((" abcdefg", nom::error::ErrorKind::IsNot)))
            );
        }
    }
}

pub fn parse_sexp(string: &str) -> Vec<SExpr> {
    let (_, sexps) = parsers::many(string).unwrap();
    sexps
}
