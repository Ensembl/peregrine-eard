# Bundles and Repeaters

To remove bundles and repeaters we need to know:

1. exactly how many times to repeat the repeater and on which variables;
2. which arguments to pass in a bundle to a call;
3. which arguments to return in a bundle from a call.

In all three cases, the answer is the values which are subsequently used. To resolve this clairvoyance, we examine the program backwards, tracking a variable back from its use to its declaration.

A variable can be used in a call:

1. directly and explicitly as an argument;
2. as part of a bundle used as an argument;
3. as a repeater argument in a repeater expression.

That variable could have been declared:

1. directly and explicitly in a let;
2. as a bundle in a let return;
3. as a repeater return.

At any point in time, tracing the call-graph backwards, we will have seen uses of variables where we have not yet seen their declarations. These will be the "unresolved" variables at this point.

1. When a variable is directly and explicitly used as an argument it is marked as unresolved (if not already so marked).

2. When a variable is directly and explicitly declared, it is unmarked as resolved.

3. When a repeater statement is found, every unresolved variable in the repeater return is unresolved in the repeater argument.

4. When a bundle return is found, we apply the algorithm within the call, marking the return prefix as containing the same unresolved variables as in the call return and working up through the function.

5. When a bundle is found in the start of a function, the unresolved variables are assigned to the bundle argument in the call.

In the process we determine:

1. In step 3, the repeated instances;
2. In step 4, the bundle variables to return;
3. In step 5, the bundle arguments to pass.
