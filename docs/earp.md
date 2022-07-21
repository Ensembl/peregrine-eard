# Earp

## Introduction

Earp is the third iteration of the style compiler language. It is used to convert data to visual elements by guiding it through three processes:

1. undoing any compression which may have been done during transmission;
2. supplying any arguments which may have been implicit and transforming any which need a new form;
3. choosing shape elements to represent the data.

Earp is not a general-purpose programming language and attempts to treat it as such will lead to frustration. It is more like:

* a workflow/pipeline language;
* or a patch or pipeline specification lagnuage a means of gluing things together as used in audio/video processing;
* a shader language as used in graphics card.

To this end it is aimed at efficient storage and processing of SIMD data as the amount of data may be large. It should be easy to extend implementations to SIMD processor instructions, multiple threads, and so on, without any programmer work. Its execution time is deterministic and stopping is guaranteed.

The simple syntax also allows many more mistakes to be picked up at compile time and should discourage the use of overly complex processing in earp files.

The language is designed to be maximal in its primitives (which are implemented in native Rust). Should some complex computation or datatype be required, it can be implemented as new functions and procedures in the interpreter for users to call rather than in earp itself.

Earp is essentially a procedural syntax for a declarative task.

## Types

Earp has a highly restircted range of types. Thiskeeps operations very fast and data compact despite the small and primitive interpreter. Atomic types are

* boolean
* string
* number (64-bit float)
* handles.

Handles can be minted by a library and passed around as opaque pointers, allowing more complex operations.

In addition to atomic types, sequences of each atomic type can be defined. Note that there are no sequences of sequences: they are "one-dimensional" only.

A sequence may be finite or infinite. A finite sequence is like an array or list in other languages. An infinite sequence is a single, endlessly repeating value. THis isuseful for specifying unchanging attributes. For example, you may have 200 rectangles, all blue, or all 8px high. Or maybe the heights or colours vary. Accepting a sequence allows this flexibility. At least one argument in such functions accepts only *finite* sequences and determines the number of items.

The length of a sequence can be important. earp provides syntax which maximises the chance of any mistakes being caught at compile time.

Sequences are stored in a sophisticated type which minimises copying and keeps mapping and filtering efficient for reptitive sequenes.

There are no structured types or enumerations in earp. Language support for features which replace structured types are detailed in later sections.

## No Branches or Loops

There are no branches or loops in earp. There are "functions" but these are essentially very hygenic macros for organising code neatly: no recursion is allowed (or possible).

Loops and recursion are generally unnesessary for the tasks for which earp was deisgned. The language natively supports very many operations to be applied to each element of a sequence. 

Conditionals, however, are the bread and butter of earp, so how is that possible without branching? Earp has *filters* and *maps*. As you can probably guess, these are inspired by MapReduce type frameworks to be super-efficient and parallelisable.

Instead of conditional blocks, earp uses conditional expressions and filters. Conditional expressions evaluate to one value or a second depending on the value of a boolean and are implemented with `if()`.

Filters are more sophisticated and generally more used. They are used for altering *some* of the elements of sequences.

## Filters

Any sequence of booleans can be used as a filter. When a filter is "applied" to another sequence of the same length, a new sequence is created which *aliases* the first. This can then be modified and the underlying sequence will change.

Take, for exmaple, a variable `tr.colour`, representing transcript colour and one called `tr.biotype` representing biotype. Now, say we wish to set the colour of every transcript with the biotype `protein_coding` to blue. In this case we 
1. create a filter of transcripts which are protein coding. Note that this creates a sequence of booleans.
2. Apply the filter to the transcript colour array to "pick out" protein-coding transcripts.
3. Now we set these values to blue, using an infinite sequence. 

```
// 1
let filter_protein_coding = ( tr.biotype == "protein_coding");

// 2
let colour_of_pc_transcripts = filter(tr.colour,filter_protein_coding);

// 3
colour_of_pc_transcripts = ["blue"...];`
```

Filters can also be applied to non-sequence types in which case they take a single boolean. If true the filter points to the value, if false to a "black-hole".

```
let value = 1;
print(value); // 1

let false_filter = filter(value,false);
false_filter = 2;
print(value); // still 1

let true_filter = filter(value,true);
true_filter = 3;
print(value); // 3
```

Of course, for this to be at all useful the boolean would be supplied by an epxression, not a constant!

## Replacing Structured Types with Maps

earp does not natively support structured types. It is not possible to represent structured types as cleanly or as efficiently as one-dimensional types. However, biology often includes values with containment relationships and so on. (Such as exons within transcripts withing genes). So how is such data represtned?

Each type of entity is represented independently in earp with flat one-dimensional strcutures. To make relationships, we use mapping sequences. A  mapping sequene is a sequence of integers the length of a child type, indicating which parent type it belongs to.

With the addition of some helpful primitives this allows operations to be performed which exploit the relationship. 

Working out the right sequence of these three functions can be fiddly, but there are only three and what each does is easy to understand: think of how complex SQL queris and maps and filters can become in other languages. These functions alone can express things like "for genes of biotype protein_coding with more than one transcript, for any exons in the first transcript to display, set the colour to black".

* `lookup` lookup value of sequenceat offsets given to make new sequence.
* `count_to_index` takes a sequence of counts and replaces them with that many of an increasing integer, staring at zero. eg `[2,1,3,0,1]` -> `[0,0,1,2,2,2,4]`: 2 zeros, 1 one, 3 twos, 0 threes and 1 four.
* `enumerate` takes a similar seqeunce of counts but for each value resets to zero. eg `[2,1,3,0,1]` -> `[0,1,0,0,1,2,0]`: a seeuqnce of length two `[0,1,`... a sequence of length 1 ...`0,`..., a sequence of length 3, ...`0,1,2`..., a sequence of length zero, and a sequence of length 1 `,0]`.

For example say you have three genes, 'X', 'Y', and 'Z', of which 'X' and 'Z' are 'major' and Y is not, and which have 1, 4, and 2 transcripts respectively, for transcripts named A..G.

```
let gn.name = ["X","Y","Z"];
let gn.major = [true,false,true];
let tr.name = ["A","B","C","D","E","F","G"];
let tr.gene = [0,1,1,1,1,2,2];
```

Say we only want the names of the transcripts in important genes:

```
// the gene indexes of the major genes
let major_gn = gn.major==true;

// a filter of transcripts at this gene index
let major_tr = lookup(major_gn,tr.gene);

// apply filter
let major_names = filter(tr.name,major_tr);
```

Now it could well be that the variable `tr.gene` wasn't transmitted directly, but instead the *number* of transcripts in each gene was transmitted. In this case `count_to_index` can be used to construct `tr.gene`.

```
let gn.tr_count = [1,4,2];
let tr.gene = count_to_index(gn.tr_count); //[0111122]
```

`enumerate` is useful when we are concerned about the n-th of the contained object, for example only the first transcript of each gene. To get a list of namesof "first" transcripts.

```
let gn.tr_count = [1,4,2];
let tr.gene = enumerate(gn.tr_count); // [0012301]
let first_tr = tr.gene == 0;    // [ttffftf]
let first_names = filter(tr.name,first_tr); // [ABF]
```

By the way, here's the full horror of how to do that earlier complex example "for genes of biotype protein_coding with more than one transcript, for any exons in the first transcript to display, set the colour to black".

```
// say you've got this from the backend
let gn.name = [etc...];
let gn.biotype = ["protein_coding",etc...];
let gn.tr_count = [1,4,2,...];
let tr.name = ["A","B","C",etc...];
let ex.colour = ["black","blue",etc...];
let tr.ex_count = [3,2,etc...];

// create the maps for ex->tr, tr->gn
let ex.tr = count_to_index(tr.ex_count);
let tr.gn = count_to_index(gn.tr_count);

/* Which genes do we want? */

// genes of biotype protein_coding
let gn.pc = gn.biotype == "protein_coding";

// genes with more than one transcript
let gn.multi_tr = gn.tr_count > 1;

// genes we want: protein coding and multi-transcript
let gn.wanted = gn.pc && gn.multi_tr;

/* Which transcripts do we want? */

// first transcripts
let tr.first = enumerate(gn.tr_count) == 0;

// transctripts of wanted genes
let tr.of_wanted_genes = lookup(gn.wanted,tr.gn);

// trs we want: first transcripts of wanted genes
let tr.to_change = tr.of_wanted_genes && tr.first;

/* Which exons do we want? */

// exons of first transcripts of wanted genes
let ex.wanted = lookup(tr.to_change,ex.tr);

/* Change! */

// change to black
let to_be_black = filter(ex.colour,ex.wanted);
to_be_black = "black";
```

Though it looks wastefully slow to separate out these operations, they are represented compactly and efficiently at each stage (for example, boolean sequences as runs of trues and falses) meaning that long "runs" of values, or skewed distributions, typical in real-world data, are handled very quickly.

## Inclusion

Earp allows file inclusion with the `include "...."` statement. The file is included just after the parse tree is generated, so must contain self-contained statements but is otherwise interpreted in the contextof the file into which it is included.

## Procedures and Functions

Earp does not have "proper" procedures and functions in the style of a traditional language. This keeps its execution linear and all the positive effects on performance, predictability, memory-management, etc, which that entails. However it does have two constructs, funcitons and procedures, which look very much like functions for the purpose of organising and compartmentalising code. This is analagous to the situation in many embedded domain languages, such as GLSL.

Procedures and functions are nearly identical. The differences are:

* *procedures* can return multiple values (or none!) but can't be used nested inside expressions, only at the top level of a let statement.
* *functions* can only return exactly one value, but can be used nested inside expressions.

Functions and procedures introduce a lexical scope for `let` statements. Values enter via arguments and exit via results. Variables in the lexical globabl scope (including via inclusion) are also visible and modfiable. Functions and procedures work as if they are hygenically lexically replaced at the poibnt of their use. They cannot call themselves transitively.

Functions and procedures contain semi-colon separated statements. The last semicolon is followed by an expression in the case of a function and a comma-separated list of expressions in the case of a procedure.

```
function verbose_add(x,y) {
    let z = x + y;
    z
}

function better_add(x,y) {
    x+y
}

procedure limit(start,end,left_limit,right_limit) {
    let start = if(left_limit<start,left_limit,start);
    let end = if(right_limit>end,right_limit,end);
    (start,end)
}
```

## Macros

Macros are evaluated at compile time but are not strictly part of the core language, being domain-spcific. They are written in Rust and transform an input syntax tree into an output tree. Unlike most macros, they are *not* primarily designed to extend the language, but to evaluate expressions using compile-time resources. A macro has the form `macro_name!()`. 

## Variable Groups

Variable names can include a dot, such as `abc.xyz`. Where such variables exist, there cannot be a variable with a name equalling a prefix split at the dot(in this case no `abc`, though `ab` and `abc.x` are fine). Otherwise dots *can* be treated as normal parts of the variable name. However, by using the prefix on its own, you can write instructions to simplify your code which work on all variables with that prefix in bulk:

1. Reducing arguments and return values from functions.
2. Allowing multiple identical expressions to be implicitly repeated.

Because there are no structured variables in earp there are typically many sequence variables all of the same length describing asingle set of entities: `gene.start`, `gene.end` `gene.name`, and so on. Without the syntax introduced here, functions would take and return vast numbers of arguments and code would be highly repetitive. 

Functions can be defined to take a prefix as an argument. In this case *all* the values with that prefix are implicitly passed or returned. Prefixes are specified with an asterisk `*`.

```
function length(*item) {
    item.end - item.start
}

gene.length = length(*gene);
transcript.length = length(*transcript);
```

If a given expression is to be applied to *all* variables with a given prefix, then prefixes can be used which repeat the expression. In this case exactly one argument and one return value maybe specified. In this case `**` is used.

```
gn.start = [1,2,3];
gn.name = ["X","Y","Z"];
tr.name = ["A","B","C",etc...];
tr.gn = [0,0,0,1,1,2]; // Map to tr

**tr_gn = lookup(**gn,tr.gn);

print(tr_gn.name); // [X,X,X,Y,Y,Z]
print(tr_gn.start); // [1,1,1,2,2,3]
```

## Length Checks

The reality of not having structured types is that there end up being a lot of variables of the same length, equalling the number of some entity (genes, transcripts, etc). Variable groups (described above) handle the syntactic implications of that but it would also be vulnerable to logic errors were there not additional checking. Ultimately, many functions take variables which must match in length. Variables and functions can be annotated in various ways which allows compile-time and runtime checks of length to catch mistakes earlier.

There are types of check which can be applied to sequences are:

* length check: `#id`. This matches the length of all variables with the same id. The variant `##id` also allows the sequence to be infinite.
* index check: `^id`. This ensures that the values in the sequence can be used as indexes into variables of length id, ie all values are integers between zero and one less than the length id.
* total check: `@id`. This ensures that the total of the values equals the length of id.

Constraints can be applied in function and procedure definitions and in let statements (and code blocks).

```
let gn.tr_count @tr #gn = [3,4,5];
let tr.gn #tr ^gn = count_to_index(gn.tr_count);
```

The compiler can deduce a lot about the length annotations itself, so the above example can also be written:

```
let gn.tr_count @tr #gn = [3,4,5];
let tr.gn = count_to_index(gn.tr_count);
```

As `tr.gn #tr ^gn` is decduced from the call to `count_to_index` on a variable of size `@tr`. In general, only "incoming" variables (for example from the nextwork) need annotating, the deductions carry through into the code.

## Opcode Commands

All instructions ultimately map down to a `code` definition. code definitions don't contain normal statements but instead various special statements defining the mapping onto bytecode. Their arguments are special placeholders for variables. `code` blocks should *not* appear in earp as written by the end user.

Opcodes are rpresented by the `opcode` statements in `code` blocks. Note that all constants must be "passed into" code blocks.

Code blocks which "modify the world", ie update state outside the interpreter, must have the "world" prefix to prevent various optimisations.

```
// Say print takes a string or number and has opcode 42

world code print(r1: string|number) {
    opcode 42, r1;
}

print("hello");
print(42);
```

Return values are treated *as arguments*. Each is allocated a register to write to.

```
// imagine that add has opcode 23
code add(r1: number, r2: number) -> (r3: number) {
    opcode 23, r3, r1, r2;
}

let x = add(3,5);
```

If a code block refers to the same register in the argument list as the return value list the register may be modified. This is used by the compiler when the argument is no longer needed. However, all code blocks with a modify form should also have a non-modify form for when this cannot be arranged.

Similarly registers in code block argument lists can be replaced by constants for special cases.

```
// imagine that add has opcode 23
code add(r1: number, r2: number) -> (r3: number) {
    opcode 23, r3, r1, r2;
}

// but opcode 24 is a modify add
code add(r1: number, r2: number) -> (r1: number) {
    opcode 24, r1, r2;
}

// and opcode 25 is an in-place incerement by one
code add(r1: number, 1) -> (r1: number) {
    opcode 25, r1;
}

let x = 2;
let y = 3;
let z = add(x,y); // 23: x is used later
let z = add(x,y); // 24: last use of x, can reuse for z
let z = add(z,1); // 25: matches pattern
```

Code blocks also include length checks in arguments and results

```
code count_to_index(r1: seq(number) @?X #?Y) -> (r2: seq(number) #?X ^?Y) {
    opcode 42, r1, r2;
}
```

In types and length checks, identifiers can be replaced with `?` to require matching.

## Versioning

We expect old versions of earp source to live much longer than interpreters which can be kept up-to-date. So the goal is to support *new* interpreters from *old* code.

During compilation there is a special include, a "patch file", depending on the target interpreter. This is a standard earp file comprising mainly code blocks (and some polyfills). One exists for each combination of source and interpreter (though they may be shared).

## Serialization syntax

A number of earp binaries can be serialized in a single output file, where they are identified by a string name. These are preceded by a header which contains:

* a magic number
* the language version
* names and offsets to each program

The body contains:

* A constant section
* An opcode section

A single reserved opcode, 0, transfers a constant into a register. All other opcodes take all-register arguments.

The whole file is serilized as cbor. The header is a hash object with string keys (for expansibility). The main opcode section is an array of integers.

## Composition

It can sometimes be useful to compose operations at the RTL level into more efficient opcodes. In general, this cannot be epxressed in the language itself. Such optimisations are identified with strings and are declared with `flag "..."` in the earp source, almost always of thepatch file.
