# Questions

In the template configuration you can define questions that Architect should ask before the files can be written to the
target. The answers to those questions are stored in the context and are therefore available in the context.

You can ask various types of questions, each with a concrete use-case.

The question names must be (possibly dot-delimited) identifiers, so that they can be used in Handlebars templates. By
specifying multiple dot-delimited identifiers you can create nested objects in the context.

Multiple occurrences of previously specified question names will overwrite prior ones without warning.

__Example__:

You define questions with the following names:

- `author.name`
- `author.email`
- `somethingElse`

Result in the context:

```json
{
  ...,
  "author": {
    "name": ...,
    "email": ...
  },
  "somethingElse": ...
}
```

## Identifier

Ask for an identifier, i.e. a String that can only consist of a limited subset of characters.

__Format__:

```regexp
^[a-zA-Z_$][a-zA-Z0-9_$]*$
```

A possible use-case for this question type can be to ask for a class name for an application's main class.

## Option

A simple yes or no question. This will store a Boolean in the context.

__Example__:

You specify `yes` for the question `insertLogStatements`.

Result in the context:

```json
{
  ...,
  "insertLogStatements": true
}
```

This question type is useful when you want to present the user with a binary choice, e.g. whether to generate
a `.gitignore` in the target directory.

## Selection

Useful where you want to offer a predefined list of values to choose from. The selected value(s)
will be created as properties in the context and have the value `true`.

You can configure a selection question to accept only one or multiple (`multi`) values to be selected.

As those values will be added as properties they must match a certain format:

```regexp
^[a-zA-Z_$][a-zA-Z0-9_$]*$
```

__Example__:

The question (`whichOnes`) defines three values: `value1`, `value2`, and `value3`. You select
`value1` and `value2`.

Result in the context:

```json
{
  ...,
  "whichOnes": {
    "value1": true,
    "value2": true
  }
}
```

A possible use-case for this type of question is to define several features a user could choose to enable when using a
template, e.g. whether to use logging statements, or to include a certain dependency.

## Text

This question type allows you to ask for arbitrary text to store in the context. No format is enforced for answers.

Possible use-cases for this question type can be to ask for a person's full name or an email address.
