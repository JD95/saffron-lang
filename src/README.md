# Saffron 

A lattice, propagation based language, half way between functional and logic programming.

# Language Design 

This section goes over the features and semantics of the language, but doesn't reflect what is actually implemented. This is very much a very early, WIP project.

## Partial Values 

In languages like Java, Python, and Javascript, you can either have a complete value
or you can know nothing about the value, ie. `null`. 

```haskell
let 
  x <- 1
in x -- 1
```

```haskell
let 
  x <- _ -- The equivalent of null
in x -- INT_MIN...INT_MAX
```

In Saffron, we aren't limited to knowing either nothing about a value
or everything, information can be partial.

```haskell
let
  x <- lt 10
in x -- INT_MIN...9
```

We can inform `x` more than once, gaining information each time so
long as all the information agrees.

```haskell
let 
  x <- lt 10
  x <- gt 5
in x -- 6...9
```

If the information is contradictory we get an error

```haskell
let 
  x <- lt 10
  x <- gt 10
in x -- Error!
```

## Structures

Values can be bundled into tuples

```haskell
let 
  x <- 1
  y <- 2
in x, y
```

Tuples can also be partial

```haskell
let 
  x <- "hello", empty 
  x <- empty, "world" 
in x, y -- "hello", "world"
```

Here we inform `x` twice, providing half of the tuple each time, ending
with a complete value.

Information from parts of a structure can propagate to it's other values:

```haskell
let
  x <- 0 
  y <- x + 1
in x, y -- (0, 1)
```

But the information flow can be in both directions at once

```haskell
let
  x <- y - 1
  y <- x + 1
in x, y -- (_, _)
```

Although we don't know the definite values of `x` or `y`, we have
setup a relationship between the two. If we gain information about
either, we also gain information about the other. We also guarentee
that if we learn information about both, the relationship must hold 
otherwise we get a contradiction.

Parts of a tuple can be accessed via indexing

```haskell
let
  pair <- 1, 2
in pair[0] -- 1
```

## Relations

Let's get on the path to start doing some useful computations with
this style of programming. First off, we can make definitions like so: 

```haskell
add = 
  let 
    x <- z - y
    y <- z - x
    z <- x + y
  in x, y, z
```

Or a bit more succinctly

```haskell
add = x, y, z where
  x <- z - y
  y <- z - x
  z <- x + y
```

Now we can use the relations between the parts of `add` to
do some calculations 

```haskell
example = result[1] where
  result <- add 
  result <- (1, _, 3)
```

Since `add` gains information about `y` from `x` and `z`,
once we inform `result`, it's able to calculate `y` as `2`. 

This could be written more succinctly like so

```haskell
example = y where
  (1, y, 3) <- add 
```

## Functions

In functional languages like Haskell, information always flows in
one direction. You provide and input, some logic happens, and you
get an output. In logic languages like Prolog, you have relations 
which can have information flowing in all directions, allowing 
functions that run "backwards". 

### How they work 

Saffron is like logic languages in that functions are really just
a specific type of relation. Everything we'd want to do with a 
function we could do with just tuples.

```haskell
addOne = input, output where 
  input <- _ 
  output <- input + 1
```

`addOne` sets up a relationship between two values `input` and `output`. 
Once we have this relationship we can gain some information about the
input.

```haskell
example = c where
  (1, a) <- addOne
  (a, b) <- addOne
  (b, c) <- addOne
``` 

Of course, working like this get's old quickly so function deinfitions
get their own syntax. 

```haskell
-- Lambda Syntax
addOne = \x -> x + 1

-- Function Syntax
addOne x = x + 1
```

`example` from before can now be written simply as

```haskell
-- Function application via a space
example = addOne (addOne (addOne 1))
```

### Purity

It's important to note that functions do not have side effects, 
which means that functions will not allow us to provide information
about their inputs. 

```haskell
illegal x = y where
  x <- 5 -- Error!
  y <- x + 1
```

## Recursive Functions and Data

Making recursive definitions in Saffron is more complicated than
other languages.  Ad-hoc recursion isn't possible in Saffron.

```haskell
let
  x <- x + 1
in x 
```

This doesn't create a self referential value, instead it just 
hits a contradiction once we gain information about `x`

```haskell
let
  x <- 0
  x <- x + 1 
in x -- Contradiction: x can't be 0 and 1!
```

If we try to create loops with functions, it also fails

```haskell
loop x = loop x -- Error!
```

Rather than allowing ad-hoc recursive definitions, Saffron provides
built-in schemes to define them.

```haskell
data List a = Cons a (List a) | Nil

length xs = 
  fold xs with 
    Nil -> 0
    Cons _ n -> n + 1
```

The `fold` scheme replaces the recursive parts of a data structure
with the results of recursive calls. `Nil` acts as the base case
since it has no recursive parts. `Cons` is a recursive case, so it's
value is the result of applying the folding logic to that part. 