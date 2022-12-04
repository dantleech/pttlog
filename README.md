Plain Text Time Logger
======================

This is a project to create a plain text time logger in Rust.

![Plain text time logger](https://user-images.githubusercontent.com/530801/204399421-aa353f56-07e7-4bb4-afb8-b85410bfd60e.gif)

- Read-only TUI for a **plain-text timesheet**.
- Supports parsing tags and tickets.
- Day, week and year views.

This is my first Rust project, it's not pretty, but it's useful 😅

Usage
-----

Create a plain text file, and add entries to it, an entry is a `YYYY-MM-DD`
date followed by time ranges and descriptions. The `@` symbol can be used to
categorise the entry:

```
2022-11-06

09:00 talking to family @personal
09:10 walking the cat @personal
10:45-12:00 working on @pttlog
15:00 pairing on that difficult task @pairing
15:23 reviewing JIRA-1234 @reviewing
16:00 @commuting to the office

2022-11-06

09:00 shouting at clouds @personal
09:30-09:40 eating breakfast @personal
10:00 ...
```

Now run `pttlogger` with the path to your timesheet:

```
$ pttlogger mytimesheet
```

Filter
------

You can filter the entries by hitting `f`. The grammar is _something_ like
this:

```
TICKET: "<ticket prefix>-.*"
TAG: alphanumeric*
CRITERIA: TICKET|TAG|OR|AND
OR: CRITERIA CRITERIA
AND: CRITERIA CRITERIA
NOT: CRITERIA
```

Show only entries tagged with `@pttlog`:

```
@pttlog
```

With `@pttlog` and part of ticket `JIRA-1234`:

```
AND @pttlog JIRA-1234
```

With `@pttlog` and part of ticket `JIRA-1234` OR anything which is not
`@lunch`:

```
OR AND @pttlog JIRA-1234 NOT @lunch
```

Installation
------------

Grab the latest release from the [releases](https://github.com/dantleech/pttlog/releases) page.

Configuration
-------------

In order for `pttlogger` to parse and report on ticket identifiers create or
modify:

```toml
[[projects]]
name="My Work Project"
ticket_prefix="JIRA-"

[[projects]]
name="Phpactor"
ticket_prefix="PHPACTOR-"
```

In the above example "ticket" identiiers such as `JIRA-1234` and `PHPACTOR-1`
will be recognized and time will be summarised for them.


Contributing
------------

Any contributions are welcome.
