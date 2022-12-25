# parsink: Parsing with weights

Most implementations of regular expression matching will only tell you
whether the pattern matched the input, and possibly where the match and
submatches occurred. But generalizing the matching algorithm just a bit
opens new possibilities for tasks we can accomplish using regular
expressions.

## Weights?

One of my favorite academic papers is titled "[A Play on Regular
Expressions][regexp-play]". It describes a non-traditional way to
implement regular expression matching, and the best part is it's written
in the style of a play. This was my first introduction to the idea of
"weighted regular expressions", and I recommend it as a relatively
gentle introduction to the idea.

[regexp-play]: https://sebfisch.github.io/haskell-regexp/

That paper gives several examples of parameterizing a regular expression
with "weights". Some of the examples are things you can do with most any
matching implementation, like simply reporting whether the input matched
or not, or returning the bounds of the leftmost-longest match. But I've
never seen another implementation that can count the number of different
ways that a regex can match a particular input. And none of these
examples require changing the core matching algorithm.

The authors developed a library, [weighted-regexp][], based on the ideas
in this paper. I once used that library to solve a tricky problem for a
client. They continued using that solution for years.

[weighted-regexp]: https://hackage.haskell.org/package/weighted-regexp

The main downside of `weighted-regexp` is that it copies the matching
state once for each character of the input, instead of updating the
state in-place. On larger regular expressions, that causes a lot of work
for the garbage collector. The library was written in idiomatic Haskell,
which I think was a good choice for presenting the ideas, but writing a
production implementation in that language would be harder.

## Comparing RE2

So let's compare the `weighted-regexp` implementation against a
production implementation of regular expression matching. There's an
excellent series of blog posts by Russ Cox, giving a history of
implementation techniques leading up to Google's [RE2][] library. The
first article is [Regular Expression Matching Can Be Simple And
Fast][regexp-fast].

[RE2]: https://github.com/google/re2
[regexp-fast]: https://swtch.com/~rsc/regexp/regexp1.html

I want to focus on the second article in the series, titled "[Regular
Expression Matching: the Virtual Machine Approach][regexp-vm]".

[regexp-vm]: https://swtch.com/~rsc/regexp/regexp2.html

To be continued...
