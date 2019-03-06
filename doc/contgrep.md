% CONTGREP(1) User Manual
% Angelos Oikonomopoulos
% March 2019

# NAME

contgrep - grep with line continuations

# SYNOPSIS

contgrep [*options*] [file...]

# DESCRIPTION

`contgrep` is a toy grep which can handle some simple cases of multi-line searching. Specifically, it can be told (by means of a regular expression) which lines to consider continuations of a previous line. Then, it can act just like grep, except that if a match is found in line with continuations (either in the leading line or its continuations), the whole set is output.

For example, in the default settings (which is tuned for log files starting with a timestamp), any line starting with whitespace is considered a continuation. So for the file

```
<timestamp> foo
  bar
<timestamp> baz
  foo
```

`contgrep -e foo file` will output all lines (i.e. two leading lines, each with one continuation line).

# OPTIONS

-e *RE*
:   Specify regular expression to search for. This is a required argument that can be specified multiple times. The given string will be compiled into a multi-line regular expression; the syntax is that of the `regex` rust crate.

-c *RE*
:   Any line which matches this regular expression is considered a continuation of the previous line. If not specified, defaults to "^\\s+". This is not compiled into a multi-line regular expression. Conflicts with `-C`.

-C *RE*
:   Any line matching this regular expression is considered a leading (i.e. not a continuation) line. Depending on the line format you're looking at, `-c` might be preferable. Only one of the two options is allowed on the command line.

-H
:   Prepend filename to every match. This is the default when searching more than one file.

-n
:   Prepend line number to every match.

# NOTES

Input is treated as arbitrary bytes. That means that it does not need
to be of a valid encoding. Conversely, unicode character classes are
not available when specifying a regular expression.

# EXIT STATUS
Same as `grep`, `contgrep` returns 0 if a match was found, 1 if no matches were found and 2 on error.
