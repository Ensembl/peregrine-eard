/*

The only complexity in typing is the empty list. Other than the empty list, we know the type of
every value and register from the moment it's created. However the empty list could be of any list
type. It would have been possible to require annotation of empty lists as to their underlying type
but that seems unnatural, expecially, at least from program syntax, the ambiguity is immediately
resolved in a call. However for the compiler, the adjacency need not be so immediate. It would also
be possible to allow these values to have a temporary wild-card type, but this is hard on the
compiler as the algorithm would be exponential (and, indeed NP-complete as 3SAT can be epxressed
with it). Though the factor is probably very small for typical programs, it would need careful
programming and complex cases to avoid independent variables creating a very sparse search-space.
Instead we take an intermediate approach of restricting slightly the kinds of code blocks which
lead to this explosion, reducing it to a linear algorithm. The prohibited signatures are expected
to be very rare except when *trying* to cause exponential explosion.

If we scan the program forward we know that each variable is either of a known type or a sequence
of a yet-to-be-determined (ytbd) type. The restrictions we make on signatures are to allow our model at
any point in the program to comprise only a set of independent (ytbd types, the internal state of
such uncertainty being a set of permitted base types. The permitted operations are removing 
permitted types from some ytbd type and merging them by taking their intersection and from that
moment onward treating the two types as a single type.

At the point of invocation of a code block (the only part of the program which can restrict types),
non-sequence types will be known precisely. We can therefore consider signatures with different
non-sequence types separately. These are known as signature groups. If within a signature group a
sequence argument always has the same specified type, it can be ignored. We require that the
remaining sequence types are partitioned into groups which admit the same sequence types.

*/