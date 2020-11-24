# Overview
**bloom_filter_simple** is a library that offers different implementations of a simple bloom filter based
on the initial ideas presented by Burton Howard Bloom:
> Burton H. Bloom. 1970. Space/time trade-offs in hash coding with allowable errors. Commun.
ACM 13, 7 (July 1970), 422–426. DOI:https://doi.org/10.1145/362686.362692

Basic description from [Wikipedia](https://en.wikipedia.org/wiki/Bloom_filter):

> A Bloom filter is a space-efficient probabilistic data structure, conceived by Burton Howard
Bloom in 1970, that is used to test whether an element is a member of a set. False positive
matches are possible, but false negatives are not – in other words, a query returns either
"possibly in set" or "definitely not in set". Elements can be added to the set, but not removed
(though this can be addressed with the counting Bloom filter variant); the more items added, the
larger the probability of false positives.

# Bloom Filter Implementations
The library offers two basic types of bloom filter implementations.
