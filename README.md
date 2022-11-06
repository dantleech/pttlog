Plain Text Time Logger
======================

This is a project to create a plain text time logger in Rust.

![Plain Text Time Logger](https://user-images.githubusercontent.com/530801/200184958-74cb3ea9-7c53-4ed1-b0f8-2fedcbdc60fb.png)

- Create a (read-only) TUI for a plain-text timesheet.
- Add tags to categorise your entries.
- Show total time for a given day.
- Show percentage of total time per entry.

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
```

Now run `pttlog`:

```
$ pttlog mytimesheet
```
