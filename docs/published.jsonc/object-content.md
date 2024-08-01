# Content Objects

## Content locations

Content can be stored in three different locations:

- `inline`: Content is stored directly in `published.jsonc`.
- `local`: Content is stored in a separate file. This should be
  your most common use case.
- `external`: Content is fetched from a URL.

### Inline content

Content can be added inline to `published.jsonc` using the `inline` property on
the content object.
This is useful for small amounts of content that do not require a separate file.

```jsonc
{
  "content": {
    "inline": {
      "as": "plaintext",
      "value": "This is some inline content.",
    },
  },
}
```

### Local content

Local content is content that is stored in a separate file and referenced in
`published.jsonc` using the `local` property on the content object.

```jsonc
{
  "content": {
    "local": {
      "source": {
        "as": "markdown",
        "value": "hi.md",
      },
    },
  },
}
```

### External content

External content is content that is fetched from a URL and referenced in
`published.jsonc` using the `external` property on the content object.

```jsonc
{
  "content": {
    "external": {
      "source": {
        "as": "markdown",
        "value": "https://example.com/content.md",
      },
    },
  },
}
```
