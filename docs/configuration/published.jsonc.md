# Publication Configuration

_Found in a Cynthia setup under `./cynthiaFiles/published.jsonc`._

This file stores a list of publications on your website.

It is structured as a list of objects:

```jsonc
[
  {
    // ...Content...
  },
  {
    // ...Content...
  },
]
```

These objects are split up in types, each type with their own
specialties and traits. The types are:

- `page`: The most common one, representing a normal page.
- `post`: Represents a blog post. These are also displayed in `postlists` and by
  default show some additional author
  information -- the kind you'll also see on Medium.
- `redirect`: Represents a redirect. When a user visits the page,
  they are redirected to the specified URL.
- `postlist`: Represents a list of posts. These are used to
  display a list of posts on the website. They also generate
  an RSS feed and are filterable by configuration.
- `draft`[^1]: A draft publication, these have a type nested inside them.

## Defining a publication

First, you need to define the type of publication you want to create.
From there on, you can start filling in the publication's metadata.

### Page

```jsonc
[
  {
    "page": {
      "id": "page-id",
      "title": "My page title!",
      "description": "This page contains a heading!",
      "content": {
        "inline": {
          "as": "markdown",
          "value": "# This is a heading!\n\nAnd that was about it...",
        },
      },
      "dates": {
        "published": 1721685763,
        "altered": 1721685763,
        // Oh, these are the same!
        // That means this page was never edited
        // since it's publication on 07/22/2024 @ 10:02pm UTC
      },
    },
  },
  // ... Other publications ...
]
```

Within a `page` object, you can define the following properties:

- `id`: The unique identifier of the page.
- `title`: The title of the page.
- `description`: A short description of the content of the page.
- `content`: A content object, see more of this in the [content objects doc](./published.jsonc/object-content.md).
- `dates`: A dates object, see more of this in the [dates objects doc](./published.jsonc/object-dates.md).
- `scene-override`: If defined, a non-default scene will be used. See [scenes](./Cynthia.toml/scenes.md).

### Post

A post publication is essentially a page publication with extra exposure.

```jsonc
[
  {
    "post": {
      "id": "post-id",
      "title": "My first post!",
      "short": "In this post I will tell you about me and my blog!",
      "content": {
        "local": {
          "as": "html",
          "value": "posts/first-post.html",
        },
      },
      "dates": {
        "altered": 1699658204,
        "published": 1689023804,
        // Been edited a few times...
      },
    },
  },
  // ... Other publications ...
]
```

Within a `page` object, you can define the following properties:

- `id`: The unique identifier of the post.
- `title`: The title of the page.
- `short`: A short description of the page.
- `category`: The category this page belongs to.
- `content`: A content object, see more of this in the [content objects doc](./published.jsonc/object-content.md).
- `dates`: A dates object, see more of this in the [dates objects doc](./published.jsonc/object-dates.md).
- `tags`: A list`[]` of tags. These can be used to quickly find a few alike posts.
- `scene-override`: If defined, a non-default scene will be used. See [scenes](./Cynthia.toml/scenes.md).

### Redirect

to-do

### PostList

to-do

### Draft[^1]

to-do

[^1]: Exists for _Cynthia-Dash_ only. See [features](../features.md) for more information.
