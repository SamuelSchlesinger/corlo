# Corlo

My experimentation with a clone of `servant` in rust. So far, I'm running
into issues around ergonomics of const generics which I've mostly been
punting on by putting that into the hands of the user by supplying traits
instead. The present issue which has made me decide to stop is around
lifetimes of functions. Basically, I've got this function which I have to
thread down into a series of boxed closures for the `HasLink` class, and
its complaining about lifetimes for quite valid reasons. I can fix this
by making the argument `f: F` in `to_link` `f: Box<dyn Fn(A) -> Link>`
instead, but this is gross and I'm tired so instead I'm putting it up in
its current state on GitHub.

Overall, this seems eminently possible and probably desirable to have in
rust, so I hope I gather the motivation to build it, but if someone else
sees this and wants to help, please let me know. It's very annoying to
have to build a server and then separately build a client, which is
currently what I'm doing in rust. Though I could use OpenAPI or something,
I'd like to write the server as I spec out the API a la `servant`, as
I believe that leads to the maximum developer productivity for writing
web APIs.
