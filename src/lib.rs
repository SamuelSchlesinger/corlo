#![feature(generic_associated_types)]

use std::{marker::PhantomData, rc::Rc};

struct Sequence<Component, Path> {
    component: Component,
    path: Path,
}

struct Or<Path1, Path2> {
    left: Path1,
    right: Path2,
}

struct Capture<Name, Ty> {
    name: Name,
    ty: Ty,
}

struct CaptureAll<Ty> {
    ty: Ty,
}

struct ReqBody<CTy, Ty> {
    cty: CTy,
    ty: Ty,
}

struct QueryParam<Name, Ty> {
    name: Name,
    ty: Ty,
}

trait HasLink {
    type MkLink<'a, A: 'a>;

    fn to_link<'a, A: 'a>(self, f: Rc<dyn Fn(Link) -> A + 'a>, link: Link) -> Self::MkLink<'a, A>;
}

#[derive(Debug, PartialEq, Eq)]
struct Link {
    segments: Vec<Escaped>,
    query_params: Vec<Param>,
    fragment: Option<String>,
}

impl Default for Link {
    fn default() -> Link {
        Link {
            segments: Vec::new(),
            query_params: Vec::new(),
            fragment: None,
        }
    }
}

impl Link {
    fn add_segment(mut self, seg: Escaped) -> Self {
        self.segments.push(seg);
        self
    }

    fn add_query_param(mut self, query_param: Param) -> Self {
        self.query_params.push(query_param);
        self
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Escaped {
    raw: String,
}

impl From<&str> for Escaped {
    fn from(s: &str) -> Self {
        Escaped {
            raw: String::from(format!("{}", urlencoding::encode(s))),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Param {
    Single(String, String),
    ArrayElem(String, String),
    Flag(String),
}

impl From<(&str, &str)> for Param {
    fn from(pair: (&str, &str)) -> Self {
        Param::Single(String::from(pair.0), String::from(pair.1))
    }
}

struct Verb<Method, StatusCode, CTy, Ty> {
    method: Method,
    status_code: StatusCode,
    cty: CTy,
    ty: Ty,
}

struct Segment<TStr> {
    tstr: TStr,
}

/// A trait for singleton types representing an individual string.
trait KnownString
where
    Self: Copy,
    Self: Default,
{
    fn as_str(self) -> &'static str;
}

struct Raw;

struct EmptyAPI;

impl<Method, StatusCode, CTy, Ty> HasLink for Verb<Method, StatusCode, CTy, Ty> {
    type MkLink<'a, A: 'a> = A;

    fn to_link<'a, A: 'a>(self, f: Rc<dyn Fn(Link) -> A + 'a>, link: Link) -> Self::MkLink<'a, A> {
        f(link)
    }
}

impl HasLink for Raw {
    type MkLink<'a, A: 'a> = A;

    fn to_link<'a, A: 'a>(self, f: Rc<dyn Fn(Link) -> A + 'a>, link: Link) -> Self::MkLink<'a, A> {
        f(link)
    }
}

impl HasLink for EmptyAPI {
    type MkLink<'a, A: 'a> = EmptyAPI;

    fn to_link<'a, A: 'a>(
        self,
        _f: Rc<dyn Fn(Link) -> A + 'a>,
        _link: Link,
    ) -> Self::MkLink<'a, A> {
        EmptyAPI
    }
}

impl<TStr: KnownString, Path: HasLink> HasLink for Sequence<Segment<TStr>, Path> {
    type MkLink<'a, A: 'a> = <Path as HasLink>::MkLink<'a, A>;

    fn to_link<'a, A: 'a>(self, f: Rc<dyn Fn(Link) -> A + 'a>, link: Link) -> Self::MkLink<'a, A> {
        let seg = Escaped::from(<TStr as Default>::default().as_str());
        self.path.to_link(f, link.add_segment(seg))
    }
}

trait ToHttpApiData {
    fn to_url_piece(&self) -> String;
}

impl<Name: KnownString, Ty: ToHttpApiData, Path: HasLink> HasLink
    for Sequence<QueryParam<Name, Ty>, Path>
where
    Path: 'static,
{
    // Oops, this is goofy
    type MkLink<'a, A: 'a> = Box<dyn FnOnce(Ty) -> <Path as HasLink>::MkLink<'a, A> + 'a>;

    fn to_link<'a, A: 'a>(self, f: Rc<dyn Fn(Link) -> A + 'a>, link: Link) -> Self::MkLink<'a, A> {
        Box::new(|ty| {
            let query_param = Param::from((
                <Name as Default>::default().as_str(),
                ty.to_url_piece().as_str(),
            ));

            self.path.to_link(f, link.add_query_param(query_param))
        })
    }
}

#[derive(Debug, Copy, Clone, Default)]
struct Foo;

impl KnownString for Foo {
    fn as_str(self) -> &'static str {
        "foo"
    }
}

#[test]
fn example_0() {
    type API = Sequence<Segment<Foo>, Verb<(), (), (), u32>>;

    let api: API = Sequence {
        component: Segment { tstr: Foo },
        path: Verb {
            method: (),
            status_code: (),
            cty: (),
            ty: 100,
        },
    };
    let l0: Link = api.to_link(Rc::new(|e| e), Default::default());
    assert_eq!(l0.segments, vec![Escaped::from("foo")]);

    struct Baloney;

    let api = Sequence {
        component: Segment { tstr: Foo },
        path: Sequence {
            component: QueryParam {
                name: Foo,
                ty: Baloney,
            },
            path: Verb {
                method: (),
                status_code: (),
                cty: (),
                ty: 100usize,
            },
        },
    };

    impl ToHttpApiData for Baloney {
        fn to_url_piece(&self) -> String {
            String::from("baloney")
        }
    }

    let l1 = api.to_link(Rc::new(|e| e), Default::default())(Baloney);

    assert_eq!(l1.segments, vec![Escaped::from("foo")]);
    assert_eq!(
        l1.query_params,
        vec![Param::Single(String::from("foo"), String::from("baloney"))]
    );
}
