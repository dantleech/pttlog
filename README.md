Plain Text Time Logger
======================

This is an educational project to create a plain text time logger in Rust.

Semantics
---------

- `2022-01-01` Start a new day entry
- `10:00 Hello World` Start a new time log with description `Hello World`
- `11:00` (nothing more) End a time log (no further activity)
- `JIRA-1234` parse ticket numbers.
- `#pairing #firefighting` parse tags.

Example
-------

Given the following text file:

```
2022-01-01

10:00 Doing something
10:30 Doing something

2022-01-02

10:00 Doing something
10:30 Doing something
15:00
```

Generate a report:

```
2022-01-01:
+-------+-------+-----------------+
| star  | hours | description     |
+-------+-------+-----------------+
| 10:00 | 0.5   | Doing something |
+-------+-------+-----------------+
| Total | 0.5   |
+-------+-------+

2022-01-02:
+-------+-------+-----------------+
| star  | hours | description     |
+-------+-------+-----------------+
| 10:00 | 0.5   | Doing something |
| 10:30 | 3.5   | Doing something |
+-------+-------+-----------------+
| Total | 4     |
+-------+-------+
```
