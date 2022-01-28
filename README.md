# `s3-deploy`

## Yeet your static site to the cloud

```shell
∫ s3-deploy help
s3-deploy 1.0
olmigs <migs@mdguerrero.com>
Deploy your static site to AWS S3

USAGE:
    s3-deploy <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help        Print this message or the help of the given subcommand(s)
    modified    Print recently modified files (< 24 hrs) in <PROJECT>
    print       Print objects in <BUCKET>
    yolo        Deploy recently modified files in <PROJECT> to <BUCKET>
∫
```

For a more philosophical dissection of a deployment workflow with this tool, [see this blog post](http://mdguerrero.com/blog).

### Requirements

Any static assets you wish to deploy from a root directory e.g. `project` must

1. live in `project/public`, and
2. be specified in an array in `project/out/public.json`.

`s3-deploy` assumes you have AWS access keys are available in your environment. See [Getting started](https://docs.aws.amazon.com/sdk-for-rust/latest/dg/getting-started.html#getting-started-step2).

### Usage

```shell
∫ s3-deploy modified -p /Local/static/assets

2 files modified recently:
   build/bundle.js
   index.html
∫ s3-deploy yolo -b example.com -p /Local/static/assets -s app

Upload success for app/build/bundle.js
   Entity tag "<REDACTED>"
Upload success for app/index.html
   Entity tag "<REDACTED>"
∫
```

### Notices

-   `Commands::Upload` is under-implemented
-   `Commands::Delete` is unimplemented

**Feel free to raise an issue for any of these, or other, concerns.**
