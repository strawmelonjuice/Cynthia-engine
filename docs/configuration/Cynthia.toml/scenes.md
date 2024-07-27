# Scenes

Scenes allow Cynthia to switch it's behaviour and themes completely for certain pages.
Scenes are defined in the `scenes` table of the configuration file.

The classic default scene is defined as follows:

```toml
[[scenes]]
name = "default"
sitename = "My Cynthia site!"

  [scenes.templates]
  post = "default"
  page = "default"
  postlist = "default"
```

## Scene configuration

`Cynthia.toml` allows you to define multiple scenes.
Each scene is defined as a table (`[[]]`) in the `scenes` array (`[[scenes]]`).
Each scene has the following fields all of which are required:

- `name`: The name of the scene. This value must be unique.
- `sitename`: The name of the site in this scene.
- `templates`:
  A table that defines the templates to use for each type of publication.

  > These templates are always placed in their respective `templates` directory,
  > this means that
  >
  > ```toml
  > post = "default"
  > ```
  >
  > refers to
  >
  > ```path
  >
  > ./CynthiaFiles/templates/posts/default.handlebars
  >
  > ```

  The following keys are supported:

  - `post`: The template to use for posts.
  - `page`: The template to use for pages.
  - `postlist`: The template to use for post lists.
