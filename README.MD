# Cynthia!(.rs)

- Generating websites
- Serving them
- Using a simple file-based content management system (CMS)

<font style="color: red; font-weight: bold">
Cynthia is currently in development, and is not yet ready for use. Cynthia is semantically versioned, which means no huge breaks should be expected between MAJOR versions. In the minor versions, however... Cynthia is not nearly done getting new features!</font>

> [!WARNING]  
> Windows support is not yet available. Cynthia is currently only supported on Linux and MacOS.
> As I don't run servers on Windows, it doesn't make sense for me to support it. If you want to run Cynthia on Windows, you can try using WSL2.
> 
>> UPDATE: If you _need_ Windows support timely, disable the `selfinit` feature while compiling Cynthia. This will remove the need for the lzma crate, which is not yet supported on Windows.

## usage

Firstly, you need to install the package. You can do this by running the following command:

```bash
cargo install cynthiaweb
```

After that, you can run the following command to generate a new website:

```bash
md abc
cd abc
cynthiaweb init
```

The initialisation wizard will ask you a few questions about your website, and then generate the necessary files for you.

You can then run the following command to serve your website:

```bash
cynthiaweb start
```

Or bind it to a service.

You'll of course need to forward Cynthia to the web, which you can do with a reverse proxy like Nginx or Apache.

More documentation will be available on <https://cynthia-docs.strawmelonjuice.com>!
