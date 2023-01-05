# Compilation Phases

1. PEG grammar generates parse tree (chars -> PT)
2. Pre-processing I (iterate until no work done) (PT->PT)
    a. apply macros
    b. apply includes
3. Pre-processing II (PT->PT)
    a. remove flags
    b. check repeaters only at top-level lets (in code calls not spotted here)
    c. replace operators with functions
4. Pre-build (PT->BT)
    a. transition data-structures
    b. rename and extract definitions leaving tombstones
    c. partition calls into func/proc/code
    d. check strict args on code calls
5. Remove repeaters (BT->BT)
    a. check in code calls
    b. remove them

- constructors
- type check
- check check
