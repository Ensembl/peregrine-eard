A type in eard has the form

```
type ::= { seq(atomic) | atomic }
atomic ::= { number | string | boolean | handle(identifier) }
```

Typing in eard is a little painful because code blocks can have multiple code block
implementations. This is essential as we ward friction-free handling of sequences. An
example of a difficult predicate to access is "first" which takes a sequence and returns
the first member if present. The type of the result depends on the type of the argument but 
that may not be kown (or even knowable). It could, for example, be the empty list. In this case,
the call "doesn't care" about the type of the result, but something may care later! On the 
other hand, we don't want to end up with one of the many NP-hard type inference algorithms.
If we were to allow arbitrary multiple implementations we would certainly have that, as you can
build 3-SAT out of the pieces.

To do so, we require the ability to type linearly, at each point each register belonging to a single, compact subspace of the overall available types according to the following scheme:

```
Any ---+----> AnyAtomic ------+----> NonHandle ---+----> Number
       |                      |                   +----> String
       |                      |                   `----> Boolean
       |                      `----> AnyHandle ---..---> Handle(X)
       |
       +----> AnySequence ----+----> NonHandle ---+----> Number
                              |                   +----> String
                              |                   `----> Boolean
                              `----> AnyHandle ---..---> Handle(X)

|---------ARITY ----------- | ------------- VARIETY ---------------|
```

Restriction only takes place at code headers and type statements. Each has multiple options which is the point at which non-linearity could be introduced. 