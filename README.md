# jidx

A lightweight json indexer.
Reads a huge (>GB) JSON File in a streaming manner and produces a tabular output (CSV, JSON, SQLite, CLI Table) in the
following
manner:

| JSONPath | Value | Offset |
|----------|-------|--------|
| .a[1].c  | 42    | 100    |
| .a[2].b  | "foo" | 200    |
| .a[3].b  | "foo" | 300    |
| .a[4].b  | "foo" | 500    |

The first column is the json path to the value mentioned in the second column. Offset ist the position in the input file
where the value begins.

Ah just kidding. I didn't had enogh time and just build the mvp.

