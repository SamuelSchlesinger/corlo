#![feature(generic_associated_types)]
#![feature(adt_const_parameters)]

use std::marker::PhantomData;

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
    type MkLink<A>;

    fn to_link<A, F: Fn(Link) -> A>(self, f: F, link: Link) -> Self::MkLink<A>;
}

struct Link {
    segments: Vec<Escaped>,
    query_params: Vec<Param>,
    fragment: Option<String>,
}

impl Link {
    fn add_segment(self, seg: Escaped) -> Self {
        self.segments.push(seg);
        self
    }

    fn add_query_param(self, query_param: Param) -> Self {
        self.query_params.push(query_param);
        self
    }
}

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
    type MkLink<A> = A;

    fn to_link<A, F: Fn(Link) -> A>(self, f: F, link: Link) -> Self::MkLink<A> {
        f(link)
    }
}

impl HasLink for Raw {
    type MkLink<A> = A;

    fn to_link<A, F: Fn(Link) -> A>(self, f: F, link: Link) -> Self::MkLink<A> {
        f(link)
    }
}

impl HasLink for EmptyAPI {
    type MkLink<A> = EmptyAPI;

    fn to_link<A, F: Fn(Link) -> A>(self, _f: F, _link: Link) -> Self::MkLink<A> {
        EmptyAPI
    }
}

impl<TStr: KnownString, Path: HasLink> HasLink for Sequence<Segment<TStr>, Path> {
    type MkLink<A> = <Path as HasLink>::MkLink<A>;

    fn to_link<A, F: Fn(Link) -> A>(self, f: F, link: Link) -> Self::MkLink<A> {
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
    type MkLink<A> = Box<dyn Fn(Ty) -> <Path as HasLink>::MkLink<A>>;

    fn to_link<A, F: Fn(Link) -> A>(self, f: F, link: Link) -> Self::MkLink<A> {
        Box::new(|ty| {
            let query_param = Param::from((
                <Name as Default>::default().as_str(),
                ty.to_url_piece().as_str(),
            ));

            self.path.to_link(f, link.add_query_param(query_param))
        })
    }
}
