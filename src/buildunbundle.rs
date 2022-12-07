/* By this stage have a plausible expression tree for each statement except that it has embedded
 * bundles and repeaters. This algorithm identifies which actual variables are used in each passing
 * of a bundle across the argument and return boundaries for procedures and functions, known as
 * bundle transits. It does this by going through the program in reverse, first captuing usage and 
 * then, when a bundle transit is found, recording that usage. The actual unbundling is done
 * elsewhere.
 * 
 * We transit the program in reverse, first capturing usages and associating them with bundle
 * transits going back through the code from the end to the start, from result to argument.
 * 
 * The locations of transits are identified by a tuple of a list of call indexes and an enum
 * identifying argument/return and its index, eg ([7,6],Arg(3)) or ([10],Return(2)).
 * 
 * Prefix variable usage for a bundle is recorded in a hashset. These are assembled into a stack
 * for each call context.
 * 
 * One non-trivial task is working out _where_ bundle transits take place as they can be implicit, 
 * ie not locally indicated by the syntax. (Repeaters are easy as they are always explicit and only
 * at the statement level).
 * 
 * 1. There is a bundle transit where a function or procedure accepts a bundle argument with an 
 * explicit "*" in the argument list. 
 * 
 * 2. There is a bundle transit when a function or procedure returns a bundle result with an
 * explicit "*" in the return list.
 * 
 * 3. There is a bundle transit out of a function or procedure when its return epxression
 * contains a function which returns a bundle. These are the *implicit* bundle returns which are
 * not directly indicated by syntax.
 *
 * An implicit bundle return in b(). Note that we cannot determine that b returns a bundle by its
 * own syntax: we only know it by examining its return expression.
 *  
 * function a() { let a.a = 1; *a }
 * function b() { a() }
 *
 * Due to implicit returns, we delegate whether or not a function accepts or returns a bundle in
 * the n-th position to the function definition, which is determined recursively.
 * 
 * Algorithm:
 *   For each statment in reverse order:
 *     1. If it is a repeater statement, copy from the lhs repeater to the rhs repeater.
 *     2. Mark as used any bundle.name uses in the lvalue.
 *     3. For each argument to the call:
 *        a. if callee confirms it is a bundle return, add a transit from the caller's bundle
 *           to a new bundle for the callee. If the caller discards, use an empty, anonymous bundle.
 *        b. in the callee, iterate through the return expression, recursing all of step 3 for
 *           functions and step 2 for bundle.names.
 *        c. iterate through the statements, recursing all of step 1.
 *        d. iterate through the arguments, for each which is a bundle add an argument transit
 *           into the caller. If the expression for the argument in the caller is a function,
 *           recurse step 3; if it is a bundle.name, step 2.
 */
