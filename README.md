
# QuickAccount Parser - A parser for a financial reporting DSL

Library for parsing a small DLS for financial reporting. The specific needs for such a DSL is
probably country specific but this DLS is based on account numbers beeing in a specific
range for each major "account group".

To be able to let users configure reports dynamically we let them specify ranges of accounts
and group them togheher and specify a label for both a header (optional) and a group.

```
Sales (
    3010..3010 => Webshop
    3010..4000 => Other sales
) => Sum sales
```

This will typically be represented in a report as:

```
SALES
Webshop        xxxx
Other sales    xxxx
-------------------
Sum sales      xxxx
===================
```

The "title" is optional:

```
(
    3010..3010 => Webshop
    3010..4000 => Other sales
) => Sum sales
```

Would (typically) be represented as:

```
Webshop        xxxx
Other sales    xxxx
-------------------
Sum sales      xxxx
===================
```

The groups can be nested to as many levels as you want (in the highly unlikely scnario that you
create nesting above a few houndred levels this might cause a stack overflow since we parse
these recursively):

```
Other costs (
    6000..6010 => Leasing
    (
        6020..6100 => Office supplies
        6100..6200 => Consumables
    ) => Sum misc. costs
) => Sum other costs
```

Would typically be:

```
OTHER COSTS
Leasing                  xxxx
  Office supplies   xxxx
  Consumables       xxxx
------------------------
Sum misc. costs          xxxx
-----------------------------
Sum other costs          xxxx
=============================
```

The full DSL looks like this

```
Sales (
    3010..3010 => Webshop
    3010..4000 => Other sales
) => Sum sales

(
    4000..5000 => Material
) => Sum material

(
    5000..5000 => Direct labor
    5010..6000 => Other labor costs
) => Sum labor costs

Other costs (
    6000..6010 => Leasing
    (
        6020..6100 => Office supplies
        6100..6200 => Consumables
    ) => Sum miscellaneous costs
) => Sum other costs
```

## Syntax tree

The DSL will be parsed into a syntax tree. Since the DSL and the syntax is so small
it's easier to just show it here. The eaxmple above will get parsed into a syntax tree
looking like this:

```rust
[
    Span {
        name: Some("Sales"),
        ranges: [
            Range {
                title: "Webshop",
                from: 3010,
                to: 3010,
            },
            Range {
                title: "Other sales",
                from: 3010,
                to: 4000,
            },
        ],
        subspans: [],
        sum_type: SumTotal(Some("Sum sales")),
    },
    Span {
        name: None,
        ranges: [
            Range {
                title: "Material",
                from: 4000,
                to: 5000,
            },
        ],
        subspans: [],
        sum_type: SumTotal(Some("Sum material")),
    },
    Span {
        name: None,
        ranges: [
            Range {
                title: "Direct labor",
                from: 5000,
                to: 5000,
            },
            Range {
                title: "Other labor costs",
                from: 5010,
                to: 6000,
            },
        ],
        subspans: [],
        sum_type: SumTotal(Some("Sum labor costs")),
    },
    Span {
        name: Some("Other costs"),
        ranges: [
            Range {
                title: "Leasing",
                from: 6000,
                to: 6010,
            },
        ],
        subspans: [
            Span {
                name: None,
                ranges: [
                    Range {
                        title: "Office supplies",
                        from: 6020,
                        to: 6100,
                    },
                    Range {
                        title: "Consumables",
                        from: 6100,
                        to: 6200,
                    },
                ],
                subspans: [],
                sum_type: SubTotal(Some("Sum miscellaneous costs")),
            },
        ],
        sum_type: SumTotal(Some("Sum other costs")),
    },
]
```

## Error reporting

The error reporting tries to mimick that of Rusts:

```rust
let test = "
(
    6000..6010 => Leasing
    (
        6020.6100 => Office Supplies
        6100..6200 => Consumables
    ) => Sum miscellaneous costs
) == Sum other costs
";

let mut parser = Parser::new(test);
match parser.parse() {
    Ok(_) => (),
    Err(e) => println!("{}", e),
}
```

Gives an error message looking like this:

```
line: 5, pos: 22
                6020.6100 => Småanskaffelser
---------------------^

ERROR: Invalid range syntax
```

## Development status

Note that while this correctly parses the example above it's not extensively tested for all
edge cases so if you use this library make sure to add your own tests and confirm that it
works the way you expect it.

